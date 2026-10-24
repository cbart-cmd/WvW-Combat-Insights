//! Parser EVTC WvW: dano, strips, bajas y curacion/barrera observada de Healing Stats.
//! En WvW el juego oculta nombre, cuenta y guild de los enemigos: solo llega su rango WvW como nombre.
//! El calculo de equipos y strips sigue al de WvW Fight Analysis para que las cifras cuadren.

use std::collections::HashMap;

#[path = "contribution.rs"]
mod contribution;
#[path = "healing.rs"]
mod healing;
use crate::nexusbase::gw2::{specialization_icons, weapon_skills};

/// Healing Stats cannot prove encounter-wide recording continuity. Even supported
/// observations are partial; absent observations must never be presented as zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HealingCoverage {
    #[default]
    Missing,
    Unsupported,
    /// The POV's recorded squad cannot be established (including subgroup 0).
    UnknownGroup,
    Partial,
    /// Reserved for independently verified recording continuity. EVTC alone
    /// cannot establish this, so this parser never emits Complete.
    Complete,
}

pub type HealingStatus = HealingCoverage;

/// Color de equipo en WvW.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Team {
    Red,
    Blue,
    Green,
    /// Identified allied team in live events without a verified color mapping.
    Own,
}

impl Team {
    pub const ALL: [Team; 3] = [Team::Red, Team::Blue, Team::Green];

    pub fn name(self) -> &'static str {
        match self {
            Team::Red => "Rojo",
            Team::Blue => "Azul",
            Team::Green => "Verde",
            Team::Own => "Nuestro equipo",
        }
    }

    pub fn rgba(self) -> [f32; 4] {
        match self {
            Team::Red => [1.0, 0.36, 0.36, 1.0],
            Team::Blue => [0.42, 0.66, 1.0, 1.0],
            Team::Green => [0.45, 0.9, 0.45, 1.0],
            Team::Own => [0.85, 0.85, 0.85, 1.0],
        }
    }
}

/// Jugador del log con lo que ha hecho durante el combate.
#[derive(Clone, Debug, Default)]
pub struct Player {
    pub name: String,
    pub account: String,
    pub profession: u32,
    pub elite: u32,
    /// Armas inferidas a partir de habilidades de arma observadas en el log.
    pub weapons: Vec<&'static str>,
    /// Subconjunto con uso acuatico verificado; una lanza terrestre no pertenece aqui.
    pub underwater_weapons: Vec<&'static str>,
    pub team: Option<Team>,
    /// Raw arcdps team identity for live hostility, independent of color metadata.
    pub live_team_id: Option<u16>,
    /// Subgrupo EVTC: 1..=15 pertenece a la escuadra; 0 no pertenece.
    pub subgroup: Option<u8>,
    /// Membership in the POV's recorded squad, not merely the same WvW color.
    pub squad_member: bool,
    /// Observed outgoing healing to recorded squad recipients (includes self/pets).
    /// Only meaningful with `healing_coverage == Partial`; never a complete total.
    pub healing: u64,
    /// Observed barrier, separately gated by `barrier_coverage`.
    pub barrier: u64,
    pub healing_coverage: HealingCoverage,
    pub barrier_coverage: HealingCoverage,
    pub damage: u64,
    pub strike_damage: u64,
    pub condi_damage: u64,
    /// Health-anchored damage-to-downs, following AxiLog's native methodology.
    pub down_contribution: u64,
    pub strips: u32,
    /// Observed full condition removals performed on recorded squad recipients.
    pub cleanses: u32,
    pub kills: u32,
    pub deaths: u32,
    pub downs: u32,
}

impl Player {
    pub(super) fn opposed(&self, other: &Player, affinity: u8) -> bool {
        match (self.live_team_id, other.live_team_id) {
            (Some(a), Some(b)) => a != b,
            _ => {
                matches!((self.team, other.team), (Some(a), Some(b)) if a != b)
                    || (affinity == 1
                        && (self.team == Some(Team::Own) || self.live_team_id.is_some()))
            }
        }
    }

    pub fn spec(&self) -> &'static str {
        spec_name(self.profession, self.elite)
    }

    pub fn class_color(&self) -> [f32; 4] {
        profession_color(self.profession)
    }

    pub fn specialization_icon(&self) -> Option<&'static [u8]> {
        specialization_icons::icon(self.profession, self.elite)
    }
}

/// Resultado de analizar un log.
#[derive(Clone, Debug, Default)]
pub struct Fight {
    pub duration_ms: u64,
    /// Observed ordinary-event time range on arcdps' monotonic clock, not Unix time.
    pub event_range: Option<(u64, u64)>,
    pub pov_team: Option<Team>,
    /// Subgrupo del jugador POV, si el log lo proporciona.
    pub pov_subgroup: Option<u8>,
    /// Hora de fin del log (segundos Unix) si arcdps la registro.
    pub end_unix: u64,
    /// Text language recorded by arcdps (independent of Windows/Nexus language).
    pub language: Option<crate::i18n::Language>,
    pub players: Vec<Player>,
    pub healing_coverage: HealingCoverage,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RosterCounts {
    /// Jugadores de cada color registrados en la tabla de agentes del log.
    pub red: usize,
    pub green: usize,
    pub blue: usize,
    /// Jugadores del mismo color que el POV, pero en otro subgrupo conocido.
    pub other_allies: Option<usize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TeamEventCounts {
    pub kills: u32,
    pub deaths: u32,
    pub downs: u32,
}

impl Fight {
    pub fn is_squad_member(&self, player: &Player) -> bool {
        player.squad_member
    }

    pub fn is_group_member(&self, player: &Player) -> bool {
        player.squad_member && player.subgroup == self.pov_subgroup
    }

    pub fn duration_secs(&self) -> f64 {
        self.duration_ms as f64 / 1000.0
    }

    pub fn dps(&self, player: &Player) -> f64 {
        let secs = self.duration_secs();
        if secs > 0.0 {
            player.damage as f64 / secs
        } else {
            0.0
        }
    }
}

pub fn roster_counts(fight: &Fight) -> RosterCounts {
    let red = fight
        .players
        .iter()
        .filter(|player| player.team == Some(Team::Red))
        .count();
    let green = fight
        .players
        .iter()
        .filter(|player| player.team == Some(Team::Green))
        .count();
    let blue = fight
        .players
        .iter()
        .filter(|player| player.team == Some(Team::Blue))
        .count();
    let (Some(pov_team), Some(pov_subgroup)) = (fight.pov_team, fight.pov_subgroup) else {
        return RosterCounts {
            red,
            green,
            blue,
            ..Default::default()
        };
    };
    let other_allies = fight
        .players
        .iter()
        .filter(|player| {
            player.team == Some(pov_team)
                && player
                    .subgroup
                    .is_some_and(|subgroup| subgroup != pov_subgroup)
        })
        .count();
    RosterCounts {
        red,
        green,
        blue,
        other_allies: Some(other_allies),
    }
}

pub fn team_event_counts(fight: &Fight, team: Team) -> TeamEventCounts {
    fight
        .players
        .iter()
        .filter(|player| player.team == Some(team))
        .fold(TeamEventCounts::default(), |mut counts, player| {
            counts.kills += player.kills;
            counts.deaths += player.deaths;
            counts.downs += player.downs;
            counts
        })
}

#[derive(Clone, Copy)]
struct Event {
    time: u64,
    src_agent: u64,
    dst_agent: u64,
    value: i32,
    buff_dmg: i32,
    skill_id: u32,
    src_instid: u16,
    dst_instid: u16,
    src_master_instid: u16,
    dst_master_instid: u16,
    affinity: u8,
    buff: u8,
    result: u8,
    is_activation: u8,
    is_buffremove: u8,
    is_statechange: u8,
    is_shields: u8,
    is_offcycle: u8,
    pad: u32,
}

const EVENT_SIZE: usize = 64;

impl Event {
    fn read(b: &[u8]) -> Self {
        let u64_at = |at: usize| u64::from_le_bytes(b[at..at + 8].try_into().unwrap());
        let u32_at = |at: usize| u32::from_le_bytes(b[at..at + 4].try_into().unwrap());
        let u16_at = |at: usize| u16::from_le_bytes(b[at..at + 2].try_into().unwrap());
        Self {
            time: u64_at(0),
            src_agent: u64_at(8),
            dst_agent: u64_at(16),
            value: u32_at(24) as i32,
            buff_dmg: u32_at(28) as i32,
            // 32: overstack_value
            skill_id: u32_at(36),
            src_instid: u16_at(40),
            dst_instid: u16_at(42),
            src_master_instid: u16_at(44),
            dst_master_instid: u16_at(46),
            affinity: b[48],
            buff: b[49],
            result: b[50],
            is_activation: b[51],
            is_buffremove: b[52],
            // 53..: ninety, fifty, moving
            is_statechange: b[56],
            is_shields: b[58],
            is_offcycle: b[59],
            pad: u32_at(60),
        }
    }
}

mod sc {
    pub const NONE: u8 = 0;
    #[cfg(test)]
    pub const CHANGE_UP: u8 = 3;
    #[cfg(test)]
    pub const CHANGE_DOWN: u8 = 5;
    #[cfg(test)]
    pub const SPAWN: u8 = 6;
    pub const LOG_START: u8 = 9;
    pub const LOG_END: u8 = 10;
    pub const POINT_OF_VIEW: u8 = 13;
    pub const LANGUAGE: u8 = 14;
    pub const ENTER_COMBAT: u8 = 1;
    pub const DESPAWN: u8 = 7;
    pub const HEALTH_UPDATE: u8 = 8;
    pub const WEAPON_SWAP: u8 = 11;
    pub const TEAM_CHANGE: u8 = 22;
    pub const ID_TO_GUID: u8 = 46;
    /// Desde 2025 arcdps registra las retiradas de boons como statechanges propios.
    pub const BUFF_REMOVE_ALL: u8 = 72;
    pub const WVW_TEAMS: u8 = 74;
    pub const EXTENSION: u8 = 40;
    pub const EXTENSION_COMBAT: u8 = 49;
    pub const ANIMATION_START: u8 = 67;
}

fn weapon_cast(ev: &Event) -> bool {
    ev.is_statechange == sc::ANIMATION_START
        || (ev.is_statechange == sc::NONE
            && ev.buff == 0
            && ev.is_buffremove == 0
            && ((1..=6).contains(&ev.is_activation) || ev.result == 11))
}

/// Only direct player weapon buttons count. Reflections retain the reflector's
/// address/master=0 but the original skill, so skill ownership is also required.
fn player_weapon(ev: &Event, player: &Player) -> Option<weapon_skills::WeaponSkill> {
    let skill = weapon_skills::metadata(ev.skill_id)?;
    (ev.src_master_instid == 0 && skill.profession == player.profession).then_some(skill)
}

fn infer_weapons(events: &[Event], players: &mut [Player], by_addr: &HashMap<u64, usize>) {
    let mut has_casts = vec![false; players.len()];
    for ev in events {
        if weapon_cast(ev)
            && let Some(&i) = by_addr.get(&ev.src_agent)
            && ev.src_master_instid == 0
        {
            has_casts[i] = true;
        }
    }
    let mut sets = vec![None; players.len()];
    for ev in events {
        let Some(&i) = by_addr.get(&ev.src_agent) else {
            continue;
        };
        if ev.is_statechange == sc::WEAPON_SWAP {
            // ArcDPS set IDs: 4/5 land, 0/1 water, 2 kit, 3 transform.
            sets[i] = Some(ev.dst_agent);
            continue;
        }
        let Some(skill) = player_weapon(ev, &players[i]) else {
            continue;
        };
        let cast = weapon_cast(ev);
        let direct_attack = ev.is_statechange == sc::NONE
            && ev.is_activation == 0
            && ev.buff == 0
            && ev.is_buffremove == 0
            && ev.dst_agent != 0
            && ev.dst_agent != ev.src_agent
            && ev.value > 0
            && is_hit(ev.result);
        // Squad/POV casts are stronger evidence than hits (including reflections).
        // Enemies often have no animations, so retain actual direct-hit evidence.
        if !(cast || (!has_casts[i] && direct_attack)) {
            continue;
        }
        // A kit/transform is not an equipped weapon, even with a recycled skill.
        if matches!(sets[i], Some(2 | 3)) {
            continue;
        }
        let weapon = skill.weapon;
        if !players[i].weapons.contains(&weapon) {
            players[i].weapons.push(weapon);
        }
        // Spear's aquatic skill palette is distinct from its NoUnderwater land
        // palette. Never infer water use from the weapon type or swap alone.
        if skill.aquatic
            && !matches!(sets[i], Some(4 | 5))
            && !players[i].underwater_weapons.contains(&weapon)
        {
            players[i].underwater_weapons.push(weapon);
        }
    }
}

const RESULT_KILLING_BLOW: u8 = 8;
const RESULT_DOWNED: u8 = 9;

/// Boons cuya retirada completa cuenta como strip (mismos que WvW Fight Analysis).
fn is_tracked_boon(id: u32) -> bool {
    matches!(
        id,
        717 | 718 | 719 | 725 | 726 | 740 | 743 | 873 | 1122 | 1187 | 26980
    )
}

fn is_condition(id: u32) -> bool {
    matches!(
        id,
        720 | 721 | 722 | 723 | 727 | 736 | 737 | 738 | 742 | 791 | 861 | 19426 | 26766 | 27705
    )
}

/// Resultados de golpe directo que hacen dano: normal, critico, glancing, interrupcion, golpe final y derribo.
fn is_hit(result: u8) -> bool {
    matches!(result, 0 | 1 | 2 | 5 | 8 | 9)
}

fn guid_hex(first: u64, last: u64) -> String {
    let mut s = String::with_capacity(32);
    for byte in first.to_le_bytes().iter().chain(last.to_le_bytes().iter()) {
        s.push_str(&format!("{byte:02X}"));
    }
    s
}

fn team_from_guid(hex: &str) -> Option<Team> {
    match hex {
        "BC8AEAEF73DC8C43B041CEDFEA4D5020" => Some(Team::Green),
        "5D22513B9498EB48944E94EC7A8DD657" => Some(Team::Red),
        "CF6F7C254FCB184CBCCE4738EADD8388" => Some(Team::Blue),
        _ => None,
    }
}

fn read_name(raw: &[u8]) -> (String, String, Option<u8>) {
    let mut parts = raw
        .split(|&c| c == 0)
        .map(|p| String::from_utf8_lossy(p).into_owned());
    let name = parts.next().unwrap_or_default();
    let account = parts.next().unwrap_or_default();
    let subgroup = parts
        .next()
        .and_then(|part| part.parse::<u8>().ok())
        .filter(|subgroup| *subgroup <= 15);
    (name, account, subgroup)
}

/// Analiza el contenido descomprimido de un `.evtc`.
/// `vs_players_only`: solo cuenta dano y strips sobre jugadores del equipo contrario.
pub fn parse(bytes: &[u8], vs_players_only: bool) -> Result<Fight, String> {
    if bytes.len() < 16 || &bytes[..4] != b"EVTC" {
        return Err("no es un log EVTC".into());
    }
    let revision = bytes[12];
    let boss_id = u16::from_le_bytes([bytes[13], bytes[14]]);
    if revision != 1 {
        return Err(format!("revision EVTC {revision} no soportada"));
    }
    if boss_id != 1 {
        return Err("no es un log de WvW".into());
    }
    let mut at = 16;
    let read_u32 = |at: usize| -> Result<u32, String> {
        Ok(u32::from_le_bytes(
            bytes
                .get(at..at + 4)
                .ok_or("log truncado")?
                .try_into()
                .unwrap(),
        ))
    };

    let agent_count = read_u32(at)? as usize;
    at += 4;
    let mut players: Vec<Player> = Vec::new();
    let mut by_addr: HashMap<u64, usize> = HashMap::new();
    for _ in 0..agent_count {
        let block = bytes.get(at..at + 96).ok_or("log truncado en agentes")?;
        at += 96;
        let addr = u64::from_le_bytes(block[..8].try_into().unwrap());
        let profession = u32::from_le_bytes(block[8..12].try_into().unwrap());
        let elite = u32::from_le_bytes(block[12..16].try_into().unwrap());
        // los NPC llevan elite 0xFFFFFFFF y los gadgets una profesion con los 16 bits altos a 0xFFFF
        if elite == 0xFFFF_FFFF || !(1..=9).contains(&profession) {
            continue;
        }
        let (name, account, subgroup) = read_name(&block[28..92]);
        // arcdps repite agentes con la misma direccion: se conserva uno solo
        if by_addr.contains_key(&addr) {
            continue;
        }
        by_addr.insert(addr, players.len());
        players.push(Player {
            name,
            account,
            profession,
            elite,
            subgroup,
            ..Default::default()
        });
    }

    let skill_count = read_u32(at)? as usize;
    at += 4 + skill_count * 68;
    let events: Vec<Event> = bytes
        .get(at..)
        .ok_or("log truncado en habilidades")?
        .chunks_exact(EVENT_SIZE)
        .map(Event::read)
        .collect();
    Ok(analyze_events(players, by_addr, events, vs_players_only))
}

fn analyze_events(
    mut players: Vec<Player>,
    by_addr: HashMap<u64, usize>,
    events: Vec<Event>,
    vs_players_only: bool,
) -> Fight {
    let initial_subgroups: Vec<_> = players.iter().map(|player| player.subgroup).collect();
    // primera pasada: equipos, punto de vista y duracion
    let mut team_ids: HashMap<u32, Team> = HashMap::new();
    for ev in &events {
        match ev.is_statechange {
            sc::WVW_TEAMS => {
                let red = (ev.dst_agent >> 32) as u32;
                let (blue, green) = (ev.value as u32, ev.buff_dmg as u32);
                for (id, team) in [(red, Team::Red), (blue, Team::Blue), (green, Team::Green)] {
                    if id != 0 {
                        team_ids.insert(id, team);
                    }
                }
            }
            sc::ID_TO_GUID if ev.skill_id != 0 => {
                if let Some(team) = team_from_guid(&guid_hex(ev.src_agent, ev.dst_agent)) {
                    team_ids.entry(ev.skill_id).or_insert(team);
                }
            }
            _ => {}
        }
    }

    let mut pov = None;
    let (mut first, mut last) = (u64::MAX, 0u64);
    let mut end_unix = 0u64;
    let mut language = None;
    for ev in &events {
        match ev.is_statechange {
            sc::NONE => {
                first = first.min(ev.time);
                last = last.max(ev.time);
            }
            sc::TEAM_CHANGE => {
                if let (Some(&i), Some(&team)) =
                    (by_addr.get(&ev.src_agent), team_ids.get(&(ev.value as u32)))
                {
                    players[i].team = Some(team);
                }
            }
            sc::ENTER_COMBAT => {
                if let (Some(&i), Ok(subgroup)) =
                    (by_addr.get(&ev.src_agent), u8::try_from(ev.dst_agent))
                    && subgroup <= 15
                {
                    players[i].subgroup = Some(subgroup);
                }
            }
            sc::POINT_OF_VIEW => pov = Some(ev.src_agent),
            sc::LANGUAGE => {
                language = u32::try_from(ev.src_agent)
                    .ok()
                    .and_then(crate::i18n::Language::from_game)
            }
            sc::LOG_END if ev.value != 0 => end_unix = ev.value as u32 as u64,
            sc::LOG_START if end_unix == 0 && ev.value != 0 => end_unix = ev.value as u32 as u64,
            _ => {}
        }
    }

    infer_weapons(&events, &mut players, &by_addr);
    contribution::collect(&events, &by_addr, &mut players);

    // segunda pasada: dano y strips; los instid se resuelven sobre la marcha para atribuir
    // al dueno el dano de mascotas, clones e invocaciones
    let mut instid: HashMap<u16, usize> = HashMap::new();
    for ev in &events {
        let remove_all = ev.is_statechange == sc::BUFF_REMOVE_ALL;
        if (ev.is_statechange != sc::NONE && !remove_all) || ev.is_activation != 0 {
            continue;
        }
        let src = by_addr.get(&ev.src_agent).copied();
        let dst = by_addr.get(&ev.dst_agent).copied();
        if let Some(i) = src
            && ev.src_instid != 0
        {
            instid.insert(ev.src_instid, i);
        }
        if let Some(i) = dst
            && ev.dst_instid != 0
        {
            instid.insert(ev.dst_instid, i);
        }
        if ev.buff == 0
            && let Some(target) = dst
        {
            match ev.result {
                RESULT_KILLING_BLOW => players[target].deaths += 1,
                RESULT_DOWNED => players[target].downs += 1,
                _ => {}
            }
        }
        let owner = |direct: Option<usize>, master: u16| {
            direct.or_else(|| {
                (master != 0)
                    .then(|| instid.get(&master).copied())
                    .flatten()
            })
        };
        let opposed = |actor: usize, target: Option<usize>| match target {
            Some(t) => players[actor].opposed(&players[t], ev.affinity),
            None => false,
        };

        if remove_all || ev.is_buffremove != 0 {
            // en las retiradas de boons src es quien lo pierde y dst quien lo quita
            if !(remove_all || ev.is_buffremove == 1)
                || !is_tracked_boon(ev.skill_id)
                || ev.value == 0
                || ev.dst_agent == 0
                || ev.src_instid == ev.dst_instid
            {
                continue;
            }
            let Some(stripper) = owner(dst, ev.dst_master_instid) else {
                continue;
            };
            if (players[stripper].team.is_none() && players[stripper].live_team_id.is_none())
                || (vs_players_only && !opposed(stripper, src))
            {
                continue;
            }
            players[stripper].strips += 1;
            continue;
        }

        let Some(attacker) = owner(src, ev.src_master_instid) else {
            continue;
        };
        if players[attacker].team.is_none() && players[attacker].live_team_id.is_none() {
            continue;
        }
        let vs_player = opposed(attacker, dst);
        if ev.buff == 0 && ev.result == RESULT_KILLING_BLOW && vs_player {
            players[attacker].kills += 1;
        }
        let (amount, condi) = match ev.buff {
            0 if is_hit(ev.result) && ev.value > 0 => (ev.value as u64, false),
            // 0 es el formato antiguo; 14..=18 incluye ticks y dano fuera de ciclo.
            1 if matches!(ev.result, 0 | 14..=18) && ev.buff_dmg > 0 => (ev.buff_dmg as u64, true),
            _ => continue,
        };
        if vs_players_only && !vs_player {
            continue;
        }
        let p = &mut players[attacker];
        p.damage += amount;
        if condi {
            p.condi_damage += amount;
        } else {
            p.strike_damage += amount;
        }
    }

    let pov_team = pov
        .and_then(|addr| by_addr.get(&addr))
        .and_then(|&i| players[i].team);
    let pov_subgroup = pov
        .and_then(|addr| by_addr.get(&addr))
        .and_then(|&i| players[i].subgroup);
    let healing_coverage =
        healing::collect(&events, &by_addr, pov, &initial_subgroups, &mut players);
    Fight {
        duration_ms: if last > first { last - first } else { 0 },
        event_range: (first <= last).then_some((first, last)),
        pov_team,
        pov_subgroup,
        end_unix,
        language,
        players,
        healing_coverage,
    }
}

pub(crate) fn live_fight(
    agents: &[(u64, Player, u16)],
    raw: &[nexus::event::arc::evtc::Event],
    pov: u64,
    vs_players_only: bool,
) -> Fight {
    let own_id = agents
        .iter()
        .find(|(id, _, _)| *id == pov)
        .map(|(_, _, team)| *team)
        .filter(|id| *id != 0);
    let squad_known = agents
        .iter()
        .find(|(id, _, _)| *id == pov)
        .is_some_and(|(_, p, _)| {
            !p.account.is_empty() && p.subgroup.is_some_and(|g| (1..=15).contains(&g))
        });
    let players = agents
        .iter()
        .map(|(_, p, team)| {
            let mut player = p.clone();
            player.live_team_id = (*team != 0).then_some(*team);
            if (own_id == player.live_team_id && own_id.is_some())
                || (squad_known
                    && !player.account.is_empty()
                    && player.subgroup.is_some_and(|g| (1..=15).contains(&g))
                    && (own_id.is_none() || player.live_team_id.is_none()))
            {
                player.team = Some(Team::Own);
            }
            player
        })
        .collect();
    let by_addr = agents
        .iter()
        .enumerate()
        .map(|(i, (id, _, _))| (*id, i))
        .collect();
    let empty = Event::read(&[0; EVENT_SIZE]);
    let mut events = vec![Event {
        src_agent: pov,
        is_statechange: sc::POINT_OF_VIEW,
        ..empty
    }];
    for (id, _, team) in agents {
        events.push(Event {
            src_agent: *id,
            value: i32::from(*team),
            is_statechange: sc::TEAM_CHANGE,
            ..empty
        });
    }
    events.extend(raw.iter().map(|ev| Event {
        time: ev.time,
        src_agent: ev.src_agent,
        dst_agent: ev.dst_agent,
        value: if ev.is_statechange == sc::TEAM_CHANGE {
            ev.dst_agent as u32 as i32
        } else {
            ev.value
        },
        buff_dmg: ev.buff_dmg,
        skill_id: ev.skill_id,
        src_instid: ev.src_instance_id,
        dst_instid: ev.dst_instance_id,
        src_master_instid: ev.src_master_instance_id,
        dst_master_instid: ev.dst_master_instance_id,
        affinity: ev.affinity,
        buff: ev.buff,
        result: ev.result,
        is_activation: ev.is_activation,
        is_buffremove: ev.is_buffremove,
        is_statechange: ev.is_statechange,
        is_shields: ev.is_shields,
        is_offcycle: ev.is_offcycle,
        pad: u32::from_le_bytes([ev.pad61, ev.pad62, ev.pad63, ev.pad64]),
    }));
    // Team IDs have no fixed color: only recorded WvWTeams/IDToGUID mappings
    // may resolve them. Never infer squad membership from color.
    analyze_events(players, by_addr, events, vs_players_only)
}

pub(crate) fn live_metadata(event: &nexus::event::arc::evtc::Event) -> bool {
    match event.is_statechange {
        sc::WVW_TEAMS => true,
        sc::ID_TO_GUID => team_from_guid(&guid_hex(event.src_agent, event.dst_agent)).is_some(),
        sc::EXTENSION => {
            event.src_agent as u32 == 0x9c9b3c99
                && [event.pad61, event.pad62, event.pad63, event.pad64] == [0; 4]
        }
        _ => false,
    }
}

/// Nombre de la especializacion (o de la profesion base) para mostrar.
pub fn spec_name(profession: u32, elite: u32) -> &'static str {
    match elite {
        5 => "Druid",
        7 => "Daredevil",
        18 => "Berserker",
        27 => "Dragonhunter",
        34 => "Reaper",
        40 => "Chronomancer",
        43 => "Scrapper",
        48 => "Tempest",
        52 => "Herald",
        55 => "Soulbeast",
        56 => "Weaver",
        57 => "Holosmith",
        58 => "Deadeye",
        59 => "Mirage",
        60 => "Scourge",
        61 => "Spellbreaker",
        62 => "Firebrand",
        63 => "Renegade",
        64 => "Harbinger",
        65 => "Willbender",
        66 => "Virtuoso",
        67 => "Catalyst",
        68 => "Bladesworn",
        69 => "Vindicator",
        70 => "Mechanist",
        71 => "Specter",
        72 => "Untamed",
        73 => "Troubadour",
        74 => "Paragon",
        75 => "Amalgam",
        76 => "Ritualist",
        77 => "Antiquary",
        78 => "Galeshot",
        79 => "Conduit",
        80 => "Evoker",
        81 => "Luminary",
        _ => match profession {
            1 => "Guardian",
            2 => "Warrior",
            3 => "Engineer",
            4 => "Ranger",
            5 => "Thief",
            6 => "Elementalist",
            7 => "Mesmer",
            8 => "Necromancer",
            9 => "Revenant",
            _ => "Desconocido",
        },
    }
}

pub fn profession_color(profession: u32) -> [f32; 4] {
    match profession {
        1 => [0.45, 0.76, 0.85, 1.0],
        2 => [1.0, 0.82, 0.40, 1.0],
        3 => [0.82, 0.61, 0.35, 1.0],
        4 => [0.55, 0.86, 0.51, 1.0],
        5 => [0.75, 0.56, 0.58, 1.0],
        6 => [0.96, 0.54, 0.53, 1.0],
        7 => [0.66, 0.46, 0.73, 1.0],
        8 => [0.32, 0.65, 0.44, 1.0],
        9 => [0.82, 0.43, 0.35, 1.0],
        _ => [1.0, 1.0, 1.0, 1.0],
    }
}

pub fn profession_icon(profession: u32) -> Option<&'static [u8]> {
    specialization_icons::icon(profession, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn damage_event(
        src: u64,
        dst: u64,
        value: i32,
        buff_dmg: i32,
        buff: u8,
        result: u8,
    ) -> [u8; EVENT_SIZE] {
        let mut event = [0; EVENT_SIZE];
        event[..8].copy_from_slice(&1_000u64.to_le_bytes());
        event[8..16].copy_from_slice(&src.to_le_bytes());
        event[16..24].copy_from_slice(&dst.to_le_bytes());
        event[24..28].copy_from_slice(&value.to_le_bytes());
        event[28..32].copy_from_slice(&buff_dmg.to_le_bytes());
        event[40..42].copy_from_slice(&(src as u16).to_le_bytes());
        event[42..44].copy_from_slice(&(dst as u16).to_le_bytes());
        event[49] = buff;
        event[50] = result;
        event
    }

    fn log_with_events(events: &[[u8; EVENT_SIZE]]) -> Vec<u8> {
        let mut bytes = b"EVTC20260929\x01\x01\x00\x00".to_vec();
        bytes.extend_from_slice(&3u32.to_le_bytes());
        for addr in 1..=3u64 {
            let mut agent = [0; 96];
            agent[..8].copy_from_slice(&addr.to_le_bytes());
            agent[8..12].copy_from_slice(&5u32.to_le_bytes());
            bytes.extend_from_slice(&agent);
        }
        bytes.extend_from_slice(&0u32.to_le_bytes());
        let mut teams = [0; EVENT_SIZE];
        teams[16..24].copy_from_slice(&(707u64 << 32).to_le_bytes());
        teams[28..32].copy_from_slice(&2778u32.to_le_bytes());
        teams[56] = sc::WVW_TEAMS;
        bytes.extend_from_slice(&teams);
        for (addr, team) in [(1, 707), (2, 2778), (3, 707)] {
            let mut event = damage_event(addr, 0, team, 0, 0, 0);
            event[56] = sc::TEAM_CHANGE;
            bytes.extend_from_slice(&event);
        }
        let mut pov = damage_event(1, 0, 0, 0, 0, 0);
        pov[56] = sc::POINT_OF_VIEW;
        bytes.extend_from_slice(&pov);
        for event in events {
            bytes.extend_from_slice(event);
        }
        bytes
    }

    fn at(time: u64, mut event: [u8; EVENT_SIZE]) -> [u8; EVENT_SIZE] {
        event[..8].copy_from_slice(&time.to_le_bytes());
        event
    }

    fn state_event(time: u64, player: u64, state: u8) -> [u8; EVENT_SIZE] {
        let mut ev = at(time, damage_event(player, 0, 0, 0, 0, 0));
        ev[56] = state;
        ev
    }

    fn health_event(time: u64, player: u64, percent_times_100: u64) -> [u8; EVENT_SIZE] {
        let mut event = state_event(time, player, sc::HEALTH_UPDATE);
        event[16..24].copy_from_slice(&percent_times_100.to_le_bytes());
        event
    }

    #[test]
    fn down_contribution_uses_99_percent_anchor_and_includes_conditions_pets_and_downing_hit() {
        let mut pet = at(9_999, damage_event(99, 2, 40, 0, 0, 0));
        pet[44..46].copy_from_slice(&1u16.to_le_bytes());
        let events = [
            health_event(7_000, 2, 9900),
            health_event(8_000, 2, 9800),
            at(4_999, damage_event(1, 2, 10, 0, 0, 0)),
            at(5_000, damage_event(1, 2, 20, 0, 0, 0)),
            at(5_001, damage_event(1, 2, 0, 30, 1, 14)),
            at(9_000, damage_event(3, 2, 25, 0, 0, 0)),
            pet,
            at(10_000, damage_event(1, 2, 50, 0, 0, RESULT_DOWNED)),
            state_event(10_000, 2, sc::CHANGE_DOWN),
            at(10_001, damage_event(1, 2, 999, 0, 0, 0)),
        ];
        let log = log_with_events(&events);
        let default = parse(&log, true).unwrap();
        assert_eq!(default.players[0].damage, 1149);
        assert_eq!(default.players[0].down_contribution, 140);
        assert_eq!(default.players[2].down_contribution, 25);
    }

    #[test]
    fn down_contribution_resets_at_2100_ms_and_does_not_infer_state_only_downs() {
        let events = [
            at(1_000, damage_event(1, 2, 10, 0, 0, 0)),
            at(2_000, damage_event(99, 2, 0, 0, 0, RESULT_DOWNED)),
            state_event(2_000, 2, sc::CHANGE_DOWN),
            at(2_100, damage_event(1, 2, 100, 0, 0, 0)),
            state_event(2_200, 2, sc::CHANGE_UP),
            at(4_099, damage_event(1, 2, 20, 0, 0, 0)),
            at(4_100, damage_event(1, 2, 30, 0, 0, 0)),
            at(4_200, damage_event(1, 2, 0, 0, 0, RESULT_DOWNED)),
            at(4_201, damage_event(1, 2, 0, 0, 0, RESULT_DOWNED)),
            at(4_400, damage_event(1, 2, 40, 0, 0, 0)),
            state_event(4_500, 2, sc::CHANGE_DOWN),
        ];
        let fight = parse(&log_with_events(&events), true).unwrap();
        assert_eq!(fight.players[0].down_contribution, 40);
    }

    #[test]
    fn downing_hit_contributes_when_state_notification_precedes_it() {
        let events = [
            at(1_000, damage_event(1, 2, 10, 0, 0, 0)),
            state_event(2_000, 2, sc::CHANGE_DOWN),
            at(2_000, damage_event(1, 2, 100, 0, 0, 0)),
            at(2_000, damage_event(1, 2, 20, 0, 0, RESULT_DOWNED)),
        ];
        let fight = parse(&log_with_events(&events), true).unwrap();
        assert_eq!(fight.players[0].down_contribution, 130);
    }

    #[test]
    fn down_contribution_requires_enemy_player_downs_even_with_unfiltered_damage() {
        let events = [
            at(1_000, damage_event(1, 3, 100, 0, 0, 0)),
            state_event(1_100, 3, sc::CHANGE_DOWN),
            at(1_200, damage_event(1, 99, 200, 0, 0, RESULT_DOWNED)),
            at(1_300, damage_event(1, 2, 300, 0, 0, 0)),
            at(1_400, damage_event(1, 2, 0, 0, 0, RESULT_KILLING_BLOW)),
            state_event(1_500, 2, sc::SPAWN),
            at(1_600, damage_event(1, 2, 400, 0, 0, 0)),
        ];
        let log = log_with_events(&events);
        assert_eq!(parse(&log, false).unwrap().players[0].damage, 1000);
        assert_eq!(parse(&log, false).unwrap().players[0].down_contribution, 0);
        let mut with_down = events.to_vec();
        with_down.push(state_event(1_700, 2, sc::CHANGE_DOWN));
        assert_eq!(
            parse(&log_with_events(&with_down), false).unwrap().players[0].down_contribution,
            0
        );
        with_down.push(at(1_700, damage_event(1, 2, 0, 0, 0, RESULT_DOWNED)));
        assert_eq!(
            parse(&log_with_events(&with_down), false).unwrap().players[0].down_contribution,
            700
        );
    }

    #[test]
    fn down_contribution_anchor_fallback_clamping_and_latest_qualifying_health() {
        let mut opening = at(0, damage_event(1, 2, 10, 0, 0, 0));
        opening[40..42].copy_from_slice(&0u16.to_le_bytes());
        let hits = [
            opening,
            at(4_999, damage_event(1, 2, 20, 0, 0, 0)),
            at(5_000, damage_event(1, 2, 30, 0, 0, 0)),
            at(9_000, damage_event(1, 2, 40, 0, 0, RESULT_DOWNED)),
        ];
        let total = |extra: &[[u8; EVENT_SIZE]]| {
            let mut events = hits.to_vec();
            events.extend_from_slice(extra);
            parse(&log_with_events(&events), true).unwrap().players[0].down_contribution
        };
        assert_eq!(total(&[]), 100);
        assert_eq!(total(&[health_event(9_001, 2, 10000)]), 100);
        assert_eq!(total(&[health_event(1_000, 2, 10000)]), 100);
        assert_eq!(total(&[health_event(7_000, 2, 9899)]), 100);
        assert_eq!(total(&[health_event(7_000, 2, 20001)]), 100);
        assert_eq!(total(&[health_event(7_000, 2, 20000)]), 70);
        assert_eq!(total(&[health_event(7_000, 2, 9900)]), 70);
        assert_eq!(
            total(&[health_event(7_000, 2, 10000), health_event(8_000, 2, 10000)]),
            40
        );
    }

    #[test]
    fn down_contribution_uses_native_health_event_predicate_independently_of_damage() {
        let mut activation = at(1_000, damage_event(1, 2, 999, 0, 0, 0));
        activation[51] = 1;
        let mut removal = at(1_000, damage_event(1, 2, 999, 0, 0, 0));
        removal[52] = 1;
        let events = [
            at(1_000, damage_event(1, 2, 10, 0, 0, 3)),
            at(1_000, damage_event(1, 2, 999, 0, 0, 10)),
            at(1_000, damage_event(1, 2, 999, 0, 0, 12)),
            at(1_000, damage_event(1, 2, 0, 20, 1, 13)),
            at(1_000, damage_event(1, 2, -10, 0, 0, 0)),
            at(1_000, damage_event(1, 3, 999, 0, 0, 0)),
            activation,
            removal,
            at(2_000, damage_event(1, 2, 0, 0, 0, RESULT_DOWNED)),
        ];
        let fight = parse(&log_with_events(&events), true).unwrap();
        assert_eq!(fight.players[0].damage, 0);
        assert_eq!(fight.players[0].down_contribution, 30);
    }

    #[test]
    fn down_contribution_resolves_recycled_masters_and_rejects_same_time_identity_conflicts() {
        let mut first_pet = at(1_500, damage_event(99, 2, 10, 0, 0, 0));
        first_pet[44..46].copy_from_slice(&1u16.to_le_bytes());
        let mut recycled = at(2_000, damage_event(3, 0, 0, 0, 0, 0));
        recycled[40..42].copy_from_slice(&1u16.to_le_bytes());
        let mut later_pet = at(2_500, damage_event(99, 2, 20, 0, 0, 0));
        later_pet[44..46].copy_from_slice(&1u16.to_le_bytes());
        let mut conflict = at(3_000, damage_event(98, 2, 999, 0, 0, 0));
        conflict[40..42].copy_from_slice(&99u16.to_le_bytes());
        conflict[44..46].copy_from_slice(&1u16.to_le_bytes());
        let registered = at(3_000, damage_event(99, 2, 30, 0, 0, 0));
        let mut extension = at(3_500, damage_event(88, 0, 0, 0, 0, 0));
        extension[40..42].copy_from_slice(&1u16.to_le_bytes());
        extension[56] = sc::EXTENSION_COMBAT;
        let mut final_pet = at(3_600, damage_event(99, 2, 40, 0, 0, 0));
        final_pet[44..46].copy_from_slice(&1u16.to_le_bytes());
        let events = [
            first_pet,
            recycled,
            later_pet,
            conflict,
            registered,
            extension,
            final_pet,
            at(4_000, damage_event(1, 2, 0, 0, 0, RESULT_DOWNED)),
        ];
        let fight = parse(&log_with_events(&events), true).unwrap();
        assert_eq!(fight.players[0].down_contribution, 10);
        assert_eq!(fight.players[2].down_contribution, 60);
    }

    fn healing_log(events: &[[u8; EVENT_SIZE]], revision: Option<u32>) -> Vec<u8> {
        let mut all = Vec::new();
        if let Some(revision) = revision {
            let mut header = [0; EVENT_SIZE];
            header[8..16].copy_from_slice(
                &(0x9c9b3c99u64 | (u64::from(revision) << 32) | (3u64 << 56)).to_le_bytes(),
            );
            header[16..19].copy_from_slice(b"2.7");
            header[56] = sc::EXTENSION;
            all.push(header);
        }
        all.extend_from_slice(events);
        let mut bytes = log_with_events(&all);
        for (i, name) in [
            b"POV\0:POV.1234\01\0".as_slice(),
            b"Enemy\0:Enemy.1234\01\0".as_slice(),
            b"Ally\0:Ally.1234\02\0".as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            bytes[20 + i * 96 + 28..20 + i * 96 + 28 + name.len()].copy_from_slice(name);
        }
        bytes
    }

    fn heal_event(src: u64, dst: u64, amount: i32, barrier: bool) -> [u8; EVENT_SIZE] {
        let mut ev = damage_event(src, dst, -amount, 0, 0, 0);
        ev[36..40].copy_from_slice(&123u32.to_le_bytes());
        ev[56] = sc::EXTENSION_COMBAT;
        ev[58] = u8::from(barrier);
        ev[59] = 0x80;
        ev[60..64].copy_from_slice(&0x9c9b3c99u32.to_le_bytes());
        ev
    }

    fn cleanse_event(recipient: u64, cleanser: u64, skill: u32, modern: bool) -> [u8; EVENT_SIZE] {
        let mut ev = damage_event(recipient, cleanser, 1000, 0, 1, 0);
        ev[36..40].copy_from_slice(&skill.to_le_bytes());
        ev[52] = 1;
        if modern {
            ev[56] = sc::BUFF_REMOVE_ALL;
        }
        ev
    }

    #[test]
    fn cleanses_count_full_conditions_not_stacks_without_healing_extension() {
        for modern in [false, true] {
            let conditions = [
                720, 721, 722, 723, 727, 736, 737, 738, 742, 791, 861, 19426, 26766, 27705,
            ];
            let mut events: Vec<_> = conditions
                .into_iter()
                .map(|skill| cleanse_event(3, 1, skill, modern))
                .collect();
            events.push(cleanse_event(1, 1, 736, modern));
            // Individual stacks and Arc's synthetic removals must not double count.
            for kind in [2, 3] {
                let mut ev = cleanse_event(3, 1, 736, modern);
                ev[52] = kind;
                events.push(ev);
            }
            for amount in [0i32, -1] {
                let mut ev = cleanse_event(3, 1, 736, modern);
                ev[24..28].copy_from_slice(&amount.to_le_bytes());
                events.push(ev);
            }
            events.push(cleanse_event(3, 1, 740, modern)); // Boon, not a condition.
            events.push(cleanse_event(2, 1, 736, modern)); // Enemy recipient.
            events.push(cleanse_event(3, 2, 736, modern)); // Enemy actor.
            events.push(cleanse_event(3, 0, 736, modern)); // Unknown actor.
            let mut activation = cleanse_event(3, 1, 736, modern);
            activation[51] = 1;
            events.push(activation);
            for filtered in [false, true] {
                let fight = parse(&healing_log(&events, None), filtered).unwrap();
                assert_eq!(fight.healing_coverage, HealingCoverage::Missing);
                assert_eq!(fight.players[0].cleanses, 15);
                assert_eq!(fight.players[0].strips, if filtered { 0 } else { 1 });
                assert_eq!(fight.players[1].cleanses, 0);
            }
        }
    }

    #[test]
    fn cleanses_resolve_pet_owners_and_use_event_time_squad_membership() {
        let mut pet = cleanse_event(3, 99, 736, true);
        pet[46..48].copy_from_slice(&1u16.to_le_bytes());
        let mut leave = damage_event(3, 0, 0, 0, 0, 0);
        leave[56] = sc::ENTER_COMBAT;
        let events = [damage_event(1, 3, 1, 0, 0, 0), pet, leave, pet];
        let fight = parse(&healing_log(&events, None), true).unwrap();
        assert_eq!(fight.players[0].cleanses, 1);
        let mut missing_pov = damage_event(99, 0, 0, 0, 0, 0);
        missing_pov[56] = sc::POINT_OF_VIEW;
        let fight = parse(&healing_log(&[missing_pov, pet], None), true).unwrap();
        assert!(fight.players.iter().all(|p| p.cleanses == 0));
    }

    #[test]
    fn healing_requires_metadata_and_tracks_each_metrics_missingness() {
        let missing = parse(&healing_log(&[], None), true).unwrap();
        assert_eq!(missing.healing_coverage, HealingCoverage::Missing);
        assert_eq!(
            missing.players[0].healing_coverage,
            HealingCoverage::Missing
        );
        for revision in [1, 2] {
            let fight = parse(
                &healing_log(&[heal_event(1, 3, 150, false)], Some(revision)),
                true,
            )
            .unwrap();
            assert_eq!(fight.healing_coverage, HealingCoverage::Partial);
            assert_eq!(fight.players[0].healing, 150);
            assert_eq!(fight.players[0].healing_coverage, HealingCoverage::Partial);
            assert_eq!(fight.players[0].barrier_coverage, HealingCoverage::Missing);
            assert_eq!(fight.players[2].healing_coverage, HealingCoverage::Missing);
        }
        for revision in [None, Some(0), Some(3), Some(0xffff)] {
            let fight = parse(
                &healing_log(&[heal_event(1, 3, 150, false)], revision),
                true,
            )
            .unwrap();
            assert_eq!(fight.healing_coverage, HealingCoverage::Unsupported);
            assert_eq!(fight.players[0].healing, 0);
            assert_eq!(
                fight.players[0].healing_coverage,
                HealingCoverage::Unsupported
            );
        }
    }

    #[test]
    fn healing_separates_barrier_indirect_healing_and_rejects_wrong_signatures() {
        let mut indirect = heal_event(1, 3, 0, false);
        indirect[49] = 1;
        indirect[28..32].copy_from_slice(&(-25i32).to_le_bytes());
        let mut wrong_signature = heal_event(1, 3, 999, false);
        wrong_signature[60..64].copy_from_slice(&42u32.to_le_bytes());
        let events = [
            heal_event(1, 3, 100, false),
            heal_event(1, 3, 70, true),
            indirect,
            wrong_signature,
        ];
        for filtered in [false, true] {
            let fight = parse(&healing_log(&events, Some(2)), filtered).unwrap();
            assert_eq!(
                (fight.players[0].healing, fight.players[0].barrier),
                (125, 70)
            );
            assert_eq!(fight.players[0].barrier_coverage, HealingCoverage::Partial);
            assert_eq!(fight.players[0].damage, 0);
        }
    }

    #[test]
    fn healing_restricts_to_recorded_pov_squad_not_allies_or_enemies() {
        let events = [heal_event(1, 3, 100, false), heal_event(1, 2, 999, false)];
        let fight = parse(&healing_log(&events, Some(2)), true).unwrap();
        assert_eq!(fight.players[0].healing, 100);
        assert!(fight.is_group_member(&fight.players[0]));
        assert!(fight.is_squad_member(&fight.players[2]));
        assert!(!fight.is_group_member(&fight.players[2]));
        assert!(!fight.is_squad_member(&fight.players[1]));
        for (i, raw) in [
            (0, b"POV\0:POV.1234\00\0".as_slice()),
            (2, b"Ally\0:Ally.1234\00\0".as_slice()),
            (2, b"Ally\0\02\0".as_slice()),
        ] {
            let mut bytes = healing_log(&events, Some(2));
            let at = 20 + i * 96 + 28;
            bytes[at..at + 64].fill(0);
            bytes[at..at + raw.len()].copy_from_slice(raw);
            let fight = parse(&bytes, true).unwrap();
            assert_eq!(fight.players[0].healing, 0);
            assert_eq!(
                fight.players[0].healing_coverage,
                if i == 0 {
                    HealingCoverage::UnknownGroup
                } else {
                    HealingCoverage::Missing
                }
            );
        }
        assert_eq!(read_name(b"Ally\0\02\0").2, Some(2));
        assert_eq!(read_name(b"Ally\0\02\0").1, "");
    }

    #[test]
    fn healing_prefers_source_peers_and_deduplicates_echoes() {
        let source = heal_event(1, 3, 100, false);
        let mut destination = source;
        destination[59] = 0x40;
        let mut legacy = heal_event(3, 1, 40, false);
        legacy[59] = 0;
        legacy[56] = sc::EXTENSION;
        let fight = parse(
            &healing_log(&[destination, source, source, legacy], Some(2)),
            true,
        )
        .unwrap();
        assert_eq!(fight.players[0].healing, 100);
        assert_eq!(fight.players[2].healing, 40);
        let fight = parse(&healing_log(&[destination], Some(2)), true).unwrap();
        assert_eq!(fight.players[0].healing, 100);
        assert_eq!(fight.players[0].healing_coverage, HealingCoverage::Partial);
    }

    #[test]
    fn healing_resolves_peer_instance_ids_and_attributes_pets() {
        let mut pet = heal_event(99, 3, 75, false);
        pet[44..46].copy_from_slice(&1u16.to_le_bytes());
        let mut peer = heal_event(900, 901, 50, true);
        peer[40..42].copy_from_slice(&1u16.to_le_bytes());
        peer[42..44].copy_from_slice(&3u16.to_le_bytes());
        let unknown = heal_event(777, 3, 999, false);
        let fight = parse(&healing_log(&[pet, peer, unknown], Some(2)), true).unwrap();
        assert_eq!(
            (fight.players[0].healing, fight.players[0].barrier),
            (75, 50)
        );
    }

    #[test]
    fn healing_ignores_activations_removals_positive_values_and_unidentified_pov() {
        let mut events = Vec::new();
        for field in [51, 52] {
            let mut event = heal_event(1, 3, 100, false);
            event[field] = 1;
            events.push(event);
        }
        let mut positive = heal_event(1, 3, 100, false);
        positive[24..28].copy_from_slice(&100i32.to_le_bytes());
        events.push(positive);
        let mut invalid_indirect = heal_event(1, 3, 100, false);
        invalid_indirect[49] = 1;
        invalid_indirect[28..32].copy_from_slice(&(-100i32).to_le_bytes());
        events.push(invalid_indirect);
        let fight = parse(&healing_log(&events, Some(2)), true).unwrap();
        assert_eq!(fight.players[0].healing_coverage, HealingCoverage::Missing);
        let mut missing_pov = damage_event(999, 0, 0, 0, 0, 0);
        missing_pov[56] = sc::POINT_OF_VIEW;
        let fight = parse(
            &healing_log(&[missing_pov, heal_event(1, 3, 100, false)], Some(2)),
            true,
        )
        .unwrap();
        assert!(!fight.players[0].squad_member);
        assert_eq!(
            fight.players[0].healing_coverage,
            HealingCoverage::UnknownGroup
        );
    }

    #[test]
    fn healing_uses_subgroup_changes_at_event_time_and_handles_i32_min() {
        let mut leave = damage_event(3, 0, 0, 0, 0, 0);
        leave[56] = sc::ENTER_COMBAT;
        let mut large = heal_event(1, 3, 1, false);
        large[24..28].copy_from_slice(&i32::MIN.to_le_bytes());
        let fight = parse(
            &healing_log(&[large, leave, heal_event(1, 3, 999, false)], Some(2)),
            true,
        )
        .unwrap();
        assert_eq!(fight.players[0].healing, 2_147_483_648);
        assert!(!fight.players[2].squad_member);
    }

    #[test]
    fn team_kills_do_not_require_positive_damage_and_ignore_damage_filter() {
        let mut pet_kill = damage_event(99, 2, 0, 0, 0, RESULT_KILLING_BLOW);
        pet_kill[44..46].copy_from_slice(&1u16.to_le_bytes());
        let events = [
            damage_event(1, 3, 1, 0, 0, 0),
            damage_event(1, 2, 0, 0, 0, RESULT_KILLING_BLOW),
            pet_kill,
            damage_event(1, 3, 0, 0, 0, RESULT_KILLING_BLOW),
            damage_event(1, 999, 1, 0, 0, RESULT_KILLING_BLOW),
        ];
        for filter in [false, true] {
            let fight = parse(&log_with_events(&events), filter).unwrap();
            assert_eq!(fight.players[0].kills, 2);
            assert_eq!(team_event_counts(&fight, Team::Red).kills, 2);
        }
    }

    #[test]
    fn reads_text_language_from_the_language_statechange() {
        for (code, language) in [
            (0, crate::i18n::Language::English),
            (2, crate::i18n::Language::French),
            (3, crate::i18n::Language::German),
            (4, crate::i18n::Language::Spanish),
        ] {
            let mut event = damage_event(code, 0, 0, 0, 0, 0);
            event[56] = sc::LANGUAGE;
            assert_eq!(
                parse(&log_with_events(&[event]), true).unwrap().language,
                Some(language)
            );
        }
        assert_eq!(parse(&log_with_events(&[]), true).unwrap().language, None);
        let mut unsupported = damage_event(5, 0, 0, 0, 0, 0);
        unsupported[56] = sc::LANGUAGE;
        assert_eq!(
            parse(&log_with_events(&[unsupported]), true)
                .unwrap()
                .language,
            None
        );
        let mut oversized = damage_event(u64::from(u32::MAX) + 5, 0, 0, 0, 0, 0);
        oversized[56] = sc::LANGUAGE;
        assert_eq!(
            parse(&log_with_events(&[oversized]), true)
                .unwrap()
                .language,
            None
        );
    }

    #[test]
    fn condition_damage_supports_legacy_ticks_and_all_current_results() {
        for result in [0, 14, 15, 16, 17, 18] {
            let bytes = log_with_events(&[
                damage_event(2, 1, 100, 0, 0, 1),
                damage_event(2, 1, 0, 250, 1, result),
            ]);
            let fight = parse(&bytes, true).unwrap();
            let enemy = &fight.players[1];
            assert_eq!(enemy.strike_damage, 100, "result {result}");
            assert_eq!(enemy.condi_damage, 250, "result {result}");
            assert_eq!(enemy.damage, 350, "result {result}");
            assert_eq!(fight.pov_team, Some(Team::Red));
        }
    }

    #[test]
    fn condition_damage_excludes_non_damage_results_and_nonpositive_values() {
        for result in 1..=19 {
            if (14..=18).contains(&result) {
                continue;
            }
            let fight = parse(
                &log_with_events(&[damage_event(2, 1, 0, 250, 1, result)]),
                true,
            )
            .unwrap();
            assert_eq!(fight.players[1].damage, 0, "result {result}");
        }
        for amount in [-250, 0] {
            let fight = parse(
                &log_with_events(&[damage_event(2, 1, 0, amount, 1, 14)]),
                true,
            )
            .unwrap();
            assert_eq!(fight.players[1].damage, 0);
        }
    }

    #[test]
    fn counts_deaths_and_downs_by_victim_team_even_without_damage() {
        let events = [
            damage_event(1, 2, 0, 0, 0, RESULT_DOWNED),
            damage_event(1, 2, 0, 0, 0, RESULT_KILLING_BLOW),
            damage_event(1, 3, 0, 0, 0, RESULT_DOWNED),
        ];
        let fight = parse(&log_with_events(&events), true).unwrap();

        assert_eq!(fight.players[1].downs, 1);
        assert_eq!(fight.players[1].deaths, 1);
        assert_eq!(fight.players[2].downs, 1);
        assert_eq!(
            team_event_counts(&fight, Team::Green),
            TeamEventCounts {
                kills: 0,
                deaths: 1,
                downs: 1,
            }
        );
        assert_eq!(
            team_event_counts(&fight, Team::Red),
            TeamEventCounts {
                kills: 1,
                deaths: 0,
                downs: 1,
            }
        );
    }

    #[test]
    fn condition_only_enemies_rank_with_pet_damage_and_existing_target_filters() {
        let mut pet = damage_event(4, 1, 0, 200, 1, 18);
        pet[44..46].copy_from_slice(&2u16.to_le_bytes());
        let bytes = log_with_events(&[
            damage_event(2, 1, 0, 250, 1, 14),
            pet,
            damage_event(2, 3, 0, 50, 1, 16),
            damage_event(2, 2, 0, 500, 1, 15),
            damage_event(2, 99, 0, 700, 1, 17),
        ]);
        let fight = parse(&bytes, true).unwrap();
        assert_eq!(fight.players[1].strike_damage, 0);
        assert_eq!(fight.players[1].condi_damage, 500);
        assert_eq!(fight.players[1].damage, 500);
        let ranks = crate::danger::rank(&fight, 2);
        assert_eq!(ranks.len(), 1);
        assert_eq!(ranks[0].team, Team::Green);
        assert_eq!(ranks[0].top_dps, vec![1]);
        assert!(ranks[0].top_strips.is_empty());
        assert_eq!(parse(&bytes, false).unwrap().players[1].damage, 1700);
    }

    #[test]
    fn team_guids() {
        assert_eq!(
            team_from_guid("5D22513B9498EB48944E94EC7A8DD657"),
            Some(Team::Red)
        );
        assert_eq!(team_from_guid("00"), None);
    }

    #[test]
    fn roster_counts_all_colors_and_other_allies() {
        let player = |team, subgroup| Player {
            team: Some(team),
            subgroup,
            ..Default::default()
        };
        let fight = Fight {
            pov_team: Some(Team::Blue),
            pov_subgroup: Some(2),
            players: vec![
                player(Team::Red, Some(1)),
                player(Team::Red, Some(3)),
                player(Team::Green, Some(1)),
                player(Team::Green, Some(2)),
                player(Team::Green, None),
                player(Team::Blue, Some(2)),
                player(Team::Blue, Some(2)),
                player(Team::Blue, Some(3)),
                player(Team::Blue, None),
            ],
            ..Default::default()
        };
        assert_eq!(
            roster_counts(&fight),
            RosterCounts {
                red: 2,
                green: 3,
                blue: 4,
                other_allies: Some(1),
            }
        );
        let mut missing_subgroup = fight.clone();
        missing_subgroup.pov_subgroup = None;
        assert_eq!(
            roster_counts(&missing_subgroup),
            RosterCounts {
                red: 2,
                green: 3,
                blue: 4,
                other_allies: None,
            }
        );
        missing_subgroup.pov_team = None;
        assert_eq!(
            roster_counts(&missing_subgroup),
            RosterCounts {
                red: 2,
                green: 3,
                blue: 4,
                other_allies: None,
            }
        );
    }

    #[test]
    fn enter_combat_statechange_supplies_the_pov_subgroup() {
        let mut enter = damage_event(1, 0, 0, 0, 0, 0);
        enter[16..24].copy_from_slice(&2u64.to_le_bytes());
        enter[56] = sc::ENTER_COMBAT;
        let fight = parse(&log_with_events(&[enter]), true).unwrap();
        assert_eq!(fight.pov_subgroup, Some(2));
    }

    #[test]
    fn agent_name_subgroup_is_optional_and_bounded() {
        assert_eq!(
            read_name(b"Personaje\0:Cuenta.1234\0 2\0"),
            ("Personaje".into(), ":Cuenta.1234".into(), None)
        );
        assert_eq!(
            read_name(b"Personaje\x00:Cuenta.1234\x002\x00"),
            ("Personaje".into(), ":Cuenta.1234".into(), Some(2))
        );
        assert_eq!(read_name(b"Personaje\x00:Cuenta.1234\x0016\x00").2, None);
    }

    #[test]
    fn spec_names() {
        assert_eq!(spec_name(1, 27), "Dragonhunter");
        assert_eq!(spec_name(8, 0), "Necromancer");
    }

    #[test]
    fn class_colors_follow_the_base_profession() {
        assert_eq!(profession_color(1), [0.45, 0.76, 0.85, 1.0]);
        assert_eq!(profession_color(9), [0.82, 0.43, 0.35, 1.0]);
        assert_eq!(profession_color(99), [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn specialization_icons_fall_back_to_profession_icons() {
        let elite = Player {
            profession: 1,
            elite: 27,
            ..Default::default()
        };
        let core = Player {
            profession: 8,
            elite: 0,
            ..Default::default()
        };
        assert!(elite.specialization_icon().is_some());
        assert!(core.specialization_icon().is_some());
    }

    #[test]
    fn weapon_skill_mapping_identifies_equipped_weapon_usage() {
        assert_eq!(weapon_skills::for_skill(5487), Some("Dagger"));
        assert_eq!(weapon_skills::for_skill(5489), Some("Dagger"));
        assert_eq!(weapon_skills::for_skill(1), None);
    }

    fn weapon_event(skill: u32, state: u8) -> [u8; EVENT_SIZE] {
        let mut event = damage_event(1, 2, 100, 0, 0, 0);
        event[36..40].copy_from_slice(&skill.to_le_bytes());
        event[56] = state;
        event
    }

    fn weapon_player(profession: u32, events: &[[u8; EVENT_SIZE]]) -> Player {
        let mut bytes = log_with_events(events);
        bytes[28..32].copy_from_slice(&profession.to_le_bytes());
        parse(&bytes, true).unwrap().players.remove(0)
    }

    #[test]
    fn mary_vai_reflections_are_not_equipped_weapons() {
        // Real 20261019-212443.zevtc: guardian Mary casts Wall of Reflection
        // (9251) four times. Reflected projectiles have HER source and master=0,
        // but retain elementalist/mesmer/necro/thief skill IDs.
        let mut events = vec![
            weapon_event(9251, sc::ANIMATION_START),
            weapon_event(9081, sc::ANIMATION_START),
            weapon_event(73055, sc::ANIMATION_START),
        ];
        for skill in [
            5491, 10273, 10596, 40184, 10289, 10290, 42304, 62510, 62560, 62607, 63066, 63182,
            63267,
        ] {
            events.push(weapon_event(skill, sc::NONE));
        }
        // Even a reflected skill of the SAME profession is not a weapon cast.
        events.push(weapon_event(9122, sc::NONE));
        let player = weapon_player(1, &events);
        assert_eq!(player.weapons, ["Greatsword", "Spear"]);
        assert!(player.underwater_weapons.is_empty());
        // Cast-less sources still reject foreign skill ownership.
        let player = weapon_player(1, &events[3..events.len() - 1]);
        assert!(player.weapons.is_empty());
    }

    #[test]
    fn enemy_direct_weapon_damage_survives_without_animations() {
        let player = weapon_player(1, &[weapon_event(9081, sc::NONE)]);
        assert_eq!(player.weapons, ["Greatsword"]);
        let mut events = vec![];
        for field in [49, 51, 52, 56] {
            let mut event = weapon_event(9081, sc::NONE);
            event[field] = if field == 51 { 7 } else { 1 };
            events.push(event);
        }
        let mut removal = weapon_event(9081, sc::NONE);
        removal[52] = 1;
        let mut pet = weapon_event(9081, sc::NONE);
        pet[44..46].copy_from_slice(&3u16.to_le_bytes());
        let mut no_damage = weapon_event(9081, sc::NONE);
        no_damage[24..28].fill(0);
        let mut buff_tick = weapon_event(9081, sc::NONE);
        buff_tick[50] = 14;
        let mut clone = weapon_event(10273, sc::NONE);
        clone[8..16].copy_from_slice(&99u64.to_le_bytes());
        clone[44..46].copy_from_slice(&1u16.to_le_bytes());
        events.extend([removal, pet, no_damage, buff_tick, clone]);
        assert!(weapon_player(1, &events).weapons.is_empty());
    }

    #[test]
    fn weapon_evidence_uses_current_and_legacy_cast_formats() {
        for (state, activation, result) in [
            (sc::ANIMATION_START, 0, 0),
            (sc::NONE, 1, 0),
            (sc::NONE, 2, 0),
            (sc::NONE, 5, 0),
            (sc::NONE, 0, 11),
        ] {
            let mut event = weapon_event(9081, state);
            event[24..28].fill(0);
            event[51] = activation;
            event[50] = result;
            assert_eq!(weapon_player(1, &[event]).weapons, ["Greatsword"]);
        }
    }

    #[test]
    fn transformations_traits_and_pets_are_not_weapon_buttons() {
        for (profession, skill) in [
            (6, 5725),  // Conjured Lightning Hammer.
            (8, 10634), // Lich form.
            (8, 38767), // Unholy Burst triggered damage.
            (4, 31318), // Celestial Avatar.
            (5, 63107), // Specter shroud.
            (1, 72104), // Radiant Fire's triggered Zealot's Flame variant.
            (1, 76572), // Luminary radiant weapon, not equipped Staff.
            (8, 73108), // Generated Soul Shards life siphon, not a spear cast.
            (9, 73149), // Blitz Mines trait on dodge, despite API weapon_type=Spear.
        ] {
            assert!(
                weapon_player(profession, &[weapon_event(skill, sc::ANIMATION_START)])
                    .weapons
                    .is_empty(),
                "{skill}"
            );
        }
    }

    #[test]
    fn weaponmaster_and_more_than_four_weapons_are_not_hidden() {
        let events: Vec<_> = [9081, 9122, 9097, 9090, 9087, 40624]
            .map(|skill| weapon_event(skill, sc::ANIMATION_START))
            .into();
        let player = weapon_player(1, &events);
        // Core Guardian agent (elite=0) may use Firebrand Axe via Weaponmaster.
        assert_eq!(
            player.weapons,
            ["Greatsword", "Staff", "Sword", "Scepter", "Shield", "Axe"]
        );
    }

    #[test]
    fn aquatic_evidence_distinguishes_land_spear_and_actual_water_use() {
        let swap = |set: u64| {
            let mut event = weapon_event(0, sc::WEAPON_SWAP);
            event[16..24].copy_from_slice(&set.to_le_bytes());
            event
        };
        for land in [4, 5] {
            let player = weapon_player(1, &[swap(land), weapon_event(73055, sc::ANIMATION_START)]);
            assert_eq!(player.weapons, ["Spear"]);
            assert!(player.underwater_weapons.is_empty());
        }
        for water in [0, 1] {
            let player = weapon_player(1, &[swap(water), weapon_event(9189, sc::ANIMATION_START)]);
            assert_eq!(player.weapons, ["Spear"]);
            assert_eq!(player.underwater_weapons, ["Spear"]);
        }
        // A distinct aquatic weapon skill is sufficient even without swaps.
        assert_eq!(
            weapon_player(1, &[weapon_event(9189, sc::NONE)]).underwater_weapons,
            ["Spear"]
        );
        // Switching to water alone does not establish any weapon use.
        assert!(weapon_player(1, &[swap(0)]).weapons.is_empty());
        for bundle in [2, 3] {
            assert!(
                weapon_player(1, &[swap(bundle), weapon_event(9081, sc::ANIMATION_START)])
                    .weapons
                    .is_empty()
            );
        }
    }
}
