//! Provisional squad-event snapshots and session-only combat history.

use crate::nexusbase::arc::{self, AgentUpdate, CombatData, evtc::Event};
use crate::{
    app::{LiveStatus, Report},
    evtc::{self, Player},
};
use nexus::event::event_consume;
use std::{
    collections::HashMap,
    sync::{
        Arc, LazyLock, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime},
};

const MAX_EVENTS: usize = 1_000_000;

#[derive(Clone)]
struct Agent {
    player: Player,
    team: u16,
}

struct Snapshot {
    agents: Vec<(u64, Player, u16)>,
    events: Vec<Event>,
    pov: u64,
    generation: u64,
    finished: bool,
    limited: bool,
}

#[derive(Default)]
struct Live {
    roster: HashMap<u64, Agent>,
    participants: HashMap<u64, Agent>,
    pov: u64,
    active: bool,
    game_combat: bool,
    pov_instance: u16,
    started: Option<SystemTime>,
    generation: u64,
    dirty: bool,
    overflow: bool,
    shown: bool,
    events: Vec<Event>,
    metadata: Vec<Event>,
    completed: Vec<Snapshot>,
}

static LIVE: LazyLock<Mutex<Live>> = LazyLock::new(|| Mutex::new(Live::default()));
static REVISION_WARNING: AtomicBool = AtomicBool::new(false);
static PLAYER_COMBAT: AtomicBool = AtomicBool::new(false);

fn empty_event() -> Event {
    Event {
        time: 0,
        src_agent: 0,
        dst_agent: 0,
        value: 0,
        buff_dmg: 0,
        overstack_value: 0,
        skill_id: 0,
        src_instance_id: 0,
        dst_instance_id: 0,
        src_master_instance_id: 0,
        dst_master_instance_id: 0,
        affinity: 0,
        buff: 0,
        result: 0,
        is_activation: 0,
        is_buffremove: 0,
        is_ninety: 0,
        is_fifty: 0,
        is_moving: 0,
        is_statechange: 0,
        is_flanking: 0,
        is_shields: 0,
        is_offcycle: 0,
        pad61: 0,
        pad62: 0,
        pad63: 0,
        pad64: 0,
    }
}

impl Live {
    fn ingest(&mut self, event: &Event, local: bool, in_combat: bool) {
        let self_event = self.pov != 0 && event.src_agent == self.pov;
        if self_event && matches!(event.is_statechange, 0..=2) && event.src_instance_id != 0 {
            self.pov_instance = event.src_instance_id;
        }
        if event.dst_agent == self.pov && event.is_statechange == 0 && event.dst_instance_id != 0 {
            self.pov_instance = event.dst_instance_id;
        }
        if self_event && event.is_statechange == 1 {
            self.game_combat = true;
        }
        if !self.active && (self.game_combat || in_combat) && self.player_interaction(event) {
            self.begin();
        }
        let recording = self.active;
        if self_event && event.is_statechange == 2 {
            self.end();
            self.game_combat = false;
        }
        if !local && recording {
            self.push(event.clone());
        }
    }

    fn player_interaction(&self, event: &Event) -> bool {
        if event.is_statechange != 0 || event.is_activation != 0 || event.is_buffremove != 0 {
            return false;
        }
        let damage = match event.buff {
            0 => event.value > 0 && matches!(event.result, 0 | 1 | 2 | 5 | 8 | 9),
            1 => event.buff_dmg > 0 && matches!(event.result, 0 | 14..=18),
            _ => false,
        };
        if !damage {
            return false;
        }
        let own = |id, master| {
            id == self.pov
                || (self.pov_instance != 0 && master == self.pov_instance)
                || self.roster.get(&id).is_some_and(|agent| {
                    self.roster.get(&self.pov).is_some_and(|pov| {
                        (pov.team != 0 && agent.team == pov.team)
                            || ((pov.team == 0 || agent.team == 0)
                                && !pov.player.account.is_empty()
                                && pov.player.subgroup.is_some_and(|g| (1..=15).contains(&g))
                                && !agent.player.account.is_empty()
                                && agent.player.subgroup.is_some_and(|g| (1..=15).contains(&g)))
                    })
                })
        };
        let enemy = if own(event.src_agent, event.src_master_instance_id) {
            event.dst_agent
        } else if own(event.dst_agent, event.dst_master_instance_id) {
            event.src_agent
        } else {
            return false;
        };
        let Some(player) = self.roster.get(&enemy) else {
            return false;
        };
        (1..=9).contains(&player.player.profession)
            && player.player.elite != u32::MAX
            && (event.affinity == 1
                || self.roster.get(&self.pov).is_some_and(|pov| {
                    pov.team != 0 && player.team != 0 && pov.team != player.team
                }))
    }

    fn begin(&mut self) {
        self.archive();
        self.generation += 1;
        self.participants = self.roster.clone();
        self.events = self.metadata.clone();
        self.active = true;
        self.started = Some(SystemTime::now());
        self.dirty = true;
        self.overflow = false;
        self.shown = false;
    }

    fn snapshot(&mut self) -> Snapshot {
        let mut agents: Vec<_> = self
            .participants
            .iter()
            .map(|(id, a)| (*id, a.player.clone(), a.team))
            .collect();
        agents.sort_by_key(|(id, _, _)| *id);
        let finished = !self.active;
        self.dirty = false;
        if finished {
            self.started = None;
        }
        Snapshot {
            agents,
            events: if finished {
                std::mem::take(&mut self.events)
            } else {
                self.events.clone()
            },
            pov: self.pov,
            generation: self.generation,
            finished,
            limited: self.overflow,
        }
    }

    fn archive(&mut self) {
        if !self.active && self.started.is_some() {
            let snapshot = self.snapshot();
            self.completed.push(snapshot);
            if self.completed.len() > crate::settings::MAX_HISTORY_SIZE {
                self.completed.remove(0);
            }
        }
    }

    fn end(&mut self) {
        if self.active {
            self.active = false;
            self.game_combat = false;
            self.dirty = true;
        }
    }

    fn clear_roster(&mut self) {
        self.end();
        self.archive();
        *self = Self {
            generation: self.generation + 1,
            completed: std::mem::take(&mut self.completed),
            ..Default::default()
        };
    }

    fn push(&mut self, event: Event) {
        if self.events.len() >= MAX_EVENTS {
            if !self.overflow {
                self.overflow = true;
                self.dirty = true;
                #[cfg(not(test))]
                nexus::log::log(
                    nexus::log::LogLevel::Warning,
                    crate::nexusbase::project::NAME,
                    "Live event limit reached; provisional data is incomplete. Waiting for the EVTC log.",
                );
            }
            return;
        }
        self.events.push(event);
        self.dirty = true;
    }

    fn update(&mut self, id: u64, agent: Agent) {
        self.roster.insert(id, agent.clone());
        if self.active {
            let subgroup = agent.player.subgroup;
            let entry = self.participants.entry(id).or_insert_with(|| {
                let mut initial = agent.clone();
                initial.player.subgroup = Some(0);
                initial
            });
            entry.player.name = agent.player.name;
            entry.player.account = agent.player.account;
            entry.player.profession = agent.player.profession;
            entry.player.elite = agent.player.elite;
            entry.team = agent.team;
            self.push(Event {
                time: self.events.last().map_or(0, |e| e.time),
                src_agent: id,
                dst_agent: u64::from(subgroup.unwrap_or(0)),
                is_statechange: 1,
                ..empty_event()
            });
        }
    }
}

fn update(payload: &AgentUpdate, leave: bool) {
    let mut live = LIVE.lock().unwrap_or_else(|e| e.into_inner());
    if payload.is_self() {
        if leave {
            live.clear_roster();
            PLAYER_COMBAT.store(false, Ordering::Relaxed);
            return;
        }
        if live.pov != 0 && live.pov != payload.id as u64 {
            live.clear_roster();
            PLAYER_COMBAT.store(false, Ordering::Relaxed);
        }
        live.pov = payload.id as u64;
    }
    live.update(
        payload.id as u64,
        Agent {
            player: Player {
                name: payload.character().unwrap_or("").to_owned(),
                account: payload.account().unwrap_or("").to_owned(),
                profession: payload.prof,
                elite: payload.elite,
                subgroup: if leave {
                    Some(0)
                } else {
                    u8::try_from(payload.subgroup).ok().filter(|g| *g <= 15)
                },
                ..Default::default()
            },
            team: payload.team,
        },
    );
}

fn combat(payload: &CombatData, local: bool) {
    if !crate::ui::in_wvw() {
        return;
    }
    if payload.rev != 1 {
        if !REVISION_WARNING.swap(true, Ordering::Relaxed) {
            nexus::log::log(
                nexus::log::LogLevel::Warning,
                crate::nexusbase::project::NAME,
                format!(
                    "Unsupported live arcdps event revision {}; using EVTC logs only.",
                    payload.rev
                ),
            );
        }
        return;
    }
    let Some(event) = payload.event() else { return };
    let mut live = LIVE.lock().unwrap_or_else(|e| e.into_inner());
    if evtc::live_metadata(event) {
        // Replace mappings by identity so map-wide metadata stays bounded.
        live.metadata.retain(|e| {
            !(e.is_statechange == event.is_statechange && e.skill_id == event.skill_id)
        });
        live.metadata.push(event.clone());
        if local && live.active {
            live.push(event.clone());
        }
    }
    for agent in [payload.src(), payload.dst()].into_iter().flatten() {
        if agent.is_self != 0 {
            live.pov = agent.id as u64;
        }
        if (1..=9).contains(&agent.prof) && agent.elite != u32::MAX {
            let id = agent.id as u64;
            let info = live
                .roster
                .entry(id)
                .and_modify(|info| {
                    if agent.team != 0 {
                        info.team = agent.team;
                    }
                    info.player.profession = agent.prof;
                    info.player.elite = agent.elite;
                    if let Some(name) = agent.name() {
                        info.player.name = name.to_owned();
                    }
                })
                .or_insert_with(|| Agent {
                    player: Player {
                        name: agent.name().unwrap_or("").to_owned(),
                        profession: agent.prof,
                        elite: agent.elite,
                        ..Default::default()
                    },
                    team: agent.team,
                })
                .clone();
            if live.active {
                let participant = live.participants.entry(id).or_insert_with(|| info.clone());
                participant.team = info.team;
                participant.player.name = info.player.name;
                participant.player.profession = info.player.profession;
                participant.player.elite = info.player.elite;
            }
        }
    }
    // Use only the squad stream for metrics, not its overlapping local copy.
    live.ingest(event, local, crate::ui::in_combat());
    PLAYER_COMBAT.store(live.active, Ordering::Relaxed);
    if event.is_statechange == 7 {
        live.roster.remove(&event.src_agent);
    }
}

pub fn subscribe() {
    arc::subscribe(
        &arc::SELF_JOIN,
        event_consume!(|p: Option<&AgentUpdate>| {
            if let Some(p) = p {
                update(p, false);
            }
        }),
    );
    arc::subscribe(
        &arc::SQUAD_JOIN,
        event_consume!(|p: Option<&AgentUpdate>| {
            if let Some(p) = p {
                update(p, false);
            }
        }),
    );
    arc::subscribe(
        &arc::SELF_LEAVE,
        event_consume!(|p: Option<&AgentUpdate>| {
            if let Some(p) = p {
                update(p, true);
            }
        }),
    );
    arc::subscribe(
        &arc::SQUAD_LEAVE,
        event_consume!(|p: Option<&AgentUpdate>| {
            if let Some(p) = p {
                update(p, true);
            }
        }),
    );
    arc::subscribe(
        &arc::COMBAT_LOCAL,
        event_consume!(|p: Option<&CombatData>| {
            if let Some(p) = p {
                combat(p, true);
            }
        }),
    );
    arc::subscribe(
        &arc::COMBAT_SQUAD,
        event_consume!(|p: Option<&CombatData>| {
            if let Some(p) = p {
                combat(p, false);
            }
        }),
    );
    arc::replay_roster();
}

pub fn reset() {
    *LIVE.lock().unwrap_or_else(|e| e.into_inner()) = Live::default();
    REVISION_WARNING.store(false, Ordering::Relaxed);
    PLAYER_COMBAT.store(false, Ordering::Relaxed);
}

pub fn in_player_combat() -> bool {
    PLAYER_COMBAT.load(Ordering::Relaxed) && crate::ui::in_combat()
}

pub fn refresh(vs_players_only: bool) {
    let snapshots = {
        let mut live = LIVE.lock().unwrap_or_else(|e| e.into_inner());
        if !crate::ui::in_combat() {
            live.end();
            PLAYER_COMBAT.store(false, Ordering::Relaxed);
        }
        let mut snapshots = std::mem::take(&mut live.completed);
        if live.dirty && live.started.is_some() {
            snapshots.push(live.snapshot());
        }
        if live.started.is_none() && snapshots.is_empty() {
            crate::app::state().live = None;
        }
        snapshots
    };
    for snapshot in snapshots {
        let start = Instant::now();
        let fight = evtc::live_fight(
            &snapshot.agents,
            &snapshot.events,
            snapshot.pov,
            vs_players_only,
        );
        let mut live = LIVE.lock().unwrap_or_else(|e| e.into_inner());
        if snapshot.finished || (live.generation == snapshot.generation && live.active) {
            let mut state = crate::app::state();
            if !snapshot.finished && !live.shown {
                live.shown = true;
                for window in &mut state.settings.windows {
                    if window.enabled && window.auto_show && window.follow_latest {
                        window.visible = true;
                    }
                }
            }
            let report = Arc::new(Report {
                path: format!("live-{}", snapshot.generation).into(),
                file_name: String::new(),
                fight,
                parse_time: start.elapsed(),
                live: Some(LiveStatus {
                    limited: snapshot.limited,
                    finished: snapshot.finished,
                }),
            });
            if snapshot.finished {
                state.store_report(report, true);
                if !live.active && live.started.is_none() {
                    state.live = None;
                }
            } else {
                state.live = Some(report);
            }
        }
    }
}

pub fn is_report(report: &Report) -> bool {
    report.live.is_some()
}

pub fn invalidate() {
    LIVE.lock().unwrap_or_else(|e| e.into_inner()).dirty = true;
}

pub const REFRESH: Duration = Duration::from_secs(1);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evtc::{HealingCoverage, Team};

    fn agent(name: &str, account: &str, group: u8, team: u16) -> Agent {
        Agent {
            player: Player {
                name: name.into(),
                account: account.into(),
                subgroup: Some(group),
                profession: 1,
                ..Default::default()
            },
            team,
        }
    }

    fn setup() -> Live {
        let mut live = Live {
            pov: 1,
            ..Default::default()
        };
        live.roster.insert(1, agent("Self", ":self.1234", 1, 707));
        live.roster.insert(2, agent("Enemy", "", 0, 2778));
        live.roster
            .insert(3, agent("Healer", ":healer.1234", 1, 707));
        live.metadata.push(Event {
            dst_agent: 707u64 << 32,
            buff_dmg: 2778,
            is_statechange: 74,
            ..empty_event()
        });
        live.begin();
        live
    }

    fn fight(live: &Live) -> evtc::Fight {
        let mut agents: Vec<_> = live
            .participants
            .iter()
            .map(|(id, a)| (*id, a.player.clone(), a.team))
            .collect();
        agents.sort_by_key(|(id, _, _)| *id);
        evtc::live_fight(&agents, &live.events, live.pov, true)
    }

    #[test]
    fn squad_stream_counts_damage_downs_strips_cleanses_and_healing_without_local_echoes() {
        let mut live = setup();
        let hit = Event {
            time: 1000,
            src_agent: 1,
            dst_agent: 2,
            value: 100,
            src_instance_id: 1,
            dst_instance_id: 2,
            ..empty_event()
        };
        live.ingest(&hit, true, true);
        live.ingest(&hit, false, true);
        live.push(Event {
            time: 2000,
            value: 200,
            result: 9,
            ..hit.clone()
        });
        live.push(Event {
            time: 2100,
            src_agent: 2,
            dst_agent: 1,
            src_instance_id: 2,
            dst_instance_id: 1,
            value: 1,
            skill_id: 717,
            is_buffremove: 1,
            ..empty_event()
        });
        live.push(Event {
            time: 2200,
            src_agent: 1,
            dst_agent: 3,
            src_instance_id: 1,
            dst_instance_id: 3,
            value: 1,
            skill_id: 720,
            is_buffremove: 1,
            ..empty_event()
        });
        let signature = 0x9c9b3c99u32;
        let pad = signature.to_le_bytes();
        live.push(Event {
            src_agent: (2u64 << 32) | u64::from(signature),
            is_statechange: 40,
            ..empty_event()
        });
        live.push(Event {
            time: 2300,
            src_instance_id: 3,
            dst_instance_id: 1,
            value: -500,
            is_statechange: 49,
            is_offcycle: 0x80,
            pad61: pad[0],
            pad62: pad[1],
            pad63: pad[2],
            pad64: pad[3],
            ..empty_event()
        });
        let fight = fight(&live);
        assert_eq!(fight.pov_team, Some(Team::Red));
        assert_eq!(fight.players[0].damage, 300);
        assert_eq!(fight.players[0].down_contribution, 300);
        assert_eq!(fight.players[0].strips, 1);
        assert_eq!(fight.players[1].downs, 1);
        assert_eq!(fight.players[2].cleanses, 1);
        assert_eq!(fight.players[2].healing, 500);
        assert_eq!(fight.players[2].healing_coverage, HealingCoverage::Partial);
        assert_eq!(fight.players[2].barrier_coverage, HealingCoverage::Missing);
        assert!(fight.players[2].squad_member);
        assert_eq!(fight.duration_ms, 1200);
    }

    #[test]
    fn sessions_reset_without_losing_roster_or_metadata_and_local_hits_are_not_metrics() {
        let mut live = setup();
        let generation = live.generation;
        live.push(Event {
            time: 1000,
            value: 10,
            ..empty_event()
        });
        live.ingest(
            &Event {
                src_agent: 1,
                is_statechange: 2,
                ..empty_event()
            },
            true,
            false,
        );
        assert!(!live.active);
        live.ingest(
            &Event {
                src_agent: 1,
                value: 10,
                ..empty_event()
            },
            true,
            false,
        );
        assert_eq!(live.generation, generation);
        let frozen = live.events.len();
        live.ingest(
            &Event {
                src_agent: 2,
                value: 999,
                ..empty_event()
            },
            false,
            false,
        );
        assert_eq!(live.events.len(), frozen);
        live.ingest(
            &Event {
                src_agent: 1,
                is_statechange: 1,
                ..empty_event()
            },
            true,
            true,
        );
        assert!(!live.active, "ENTER_COMBAT alone may be NPC-only");
        assert_eq!(live.generation, generation);
        live.ingest(
            &Event {
                src_agent: 1,
                dst_agent: 2,
                value: 100,
                ..empty_event()
            },
            true,
            true,
        );
        assert!(live.active);
        assert_eq!(live.generation, generation + 1);
        assert_eq!(live.events.len(), 1);
        assert_eq!(live.participants.len(), 3);
        live.ingest(
            &Event {
                src_agent: 1,
                is_statechange: 1,
                ..empty_event()
            },
            true,
            true,
        );
        assert_eq!(live.generation, generation + 1);
        live.ingest(
            &Event {
                src_agent: 1,
                value: 900,
                ..empty_event()
            },
            true,
            true,
        );
        assert_eq!(live.events.len(), 1);
    }

    #[test]
    fn joins_and_leaves_are_temporal_not_retroactive_and_unknown_colors_are_not_guessed() {
        let mut live = setup();
        live.update(3, agent("Healer", ":healer.1234", 0, 707));
        assert_eq!(live.participants[&3].player.subgroup, Some(1));
        assert_eq!(live.roster[&3].player.subgroup, Some(0));
        assert!(!fight(&live).players[2].squad_member);
        live.update(3, agent("Healer", ":healer.1234", 2, 707));
        assert!(fight(&live).players[2].squad_member);
        live.events.clear();
        assert_eq!(fight(&live).pov_team, Some(Team::Own));
        assert_eq!(fight(&live).players[1].team, None);
        assert!(
            fight(&live)
                .players
                .iter()
                .all(|p| p.healing_coverage == HealingCoverage::Missing)
        );
    }

    #[test]
    fn short_fights_and_map_changes_archive_before_the_next_refresh() {
        let mut live = setup();
        live.push(Event {
            time: 1000,
            ..empty_event()
        });
        live.push(Event {
            time: 2000,
            ..empty_event()
        });
        let first = live.generation;
        live.end();
        assert!(live.dirty, "Mumble exit alone must finalize the combat");
        live.begin();
        assert_eq!(live.completed.len(), 1);
        assert_eq!(live.completed[0].generation, first);
        assert!(live.completed[0].finished);
        assert_eq!(live.completed[0].events.len(), 3);
        assert_eq!(live.generation, first + 1);
        live.clear_roster();
        assert_eq!(live.completed.len(), 2);
        assert!(live.started.is_none());
        assert!(live.events.is_empty());
        assert!(live.roster.is_empty());
        assert_eq!(live.completed[0].agents.len(), 3);
        assert!(!live.active);
    }

    #[test]
    fn overflow_is_explicit_and_does_not_evict_earlier_events_or_grow_memory() {
        let mut live = Live {
            events: vec![empty_event(); MAX_EVENTS],
            ..Default::default()
        };
        live.push(empty_event());
        assert!(live.overflow);
        assert!(live.dirty);
        assert_eq!(live.events.len(), MAX_EVENTS);
    }

    #[test]
    fn npc_only_and_allied_events_do_not_replace_the_previous_fight() {
        let mut live = setup();
        live.active = false;
        live.game_combat = false;
        let generation = live.generation;
        let previous_events = live.events.len();
        live.ingest(
            &Event {
                src_agent: 1,
                src_instance_id: 10,
                is_statechange: 1,
                ..empty_event()
            },
            true,
            true,
        );
        for (src, dst) in [(1, 99), (99, 1), (1, 3)] {
            live.ingest(
                &Event {
                    src_agent: src,
                    dst_agent: dst,
                    value: 100,
                    ..empty_event()
                },
                false,
                true,
            );
        }
        assert!(!live.active);
        assert_eq!(live.generation, generation);
        assert_eq!(live.events.len(), previous_events);
        let incoming = Event {
            src_agent: 2,
            dst_agent: 1,
            value: 100,
            ..empty_event()
        };
        live.ingest(&incoming, false, true);
        assert!(live.active);
        assert_eq!(live.generation, generation + 1);
        assert_eq!(live.events.len(), live.metadata.len() + 1);
        live.ingest(
            &Event {
                src_agent: 1,
                dst_agent: 99,
                value: 100,
                ..empty_event()
            },
            false,
            true,
        );
        assert!(live.active, "NPCs after PvP do not end the fight");
        live.ingest(
            &Event {
                src_agent: 1,
                is_statechange: 2,
                ..empty_event()
            },
            true,
            false,
        );
        assert!(!live.active);
    }

    #[test]
    fn allied_pvp_starts_live_for_a_healer_but_not_outside_own_combat() {
        let hit = Event {
            time: 1000,
            src_agent: 3,
            dst_agent: 2,
            value: 100,
            ..empty_event()
        };
        let mut live = setup();
        live.active = false;
        live.game_combat = false;
        let generation = live.generation;
        live.ingest(&hit, false, false);
        assert!(
            !live.active,
            "Other players fighting must not start our combat"
        );
        live.ingest(&hit, false, true);
        assert!(live.active);
        assert_eq!(live.generation, generation + 1);
        assert_eq!(fight(&live).players[2].damage, 100);
        live.ingest(
            &Event {
                src_agent: 1,
                is_statechange: 2,
                ..empty_event()
            },
            true,
            false,
        );
        assert!(!live.active);
        assert!(live.dirty, "Local exit must publish the final snapshot");

        live.roster.get_mut(&1).unwrap().team = 0;
        live.roster.get_mut(&3).unwrap().team = 0;
        live.ingest(&Event { affinity: 1, ..hit }, false, true);
        assert!(
            live.active,
            "Registered squad PvP does not require team colors"
        );
    }

    #[test]
    fn player_trigger_requires_damage_and_hostility_and_accepts_owned_pets() {
        let mut live = setup();
        live.pov_instance = 10;
        let hit = Event {
            src_agent: 1,
            dst_agent: 2,
            value: 100,
            ..empty_event()
        };
        assert!(live.player_interaction(&hit));
        assert!(!live.player_interaction(&Event {
            value: 0,
            ..hit.clone()
        }));
        assert!(!live.player_interaction(&Event {
            is_activation: 1,
            ..hit.clone()
        }));
        assert!(!live.player_interaction(&Event {
            is_buffremove: 1,
            ..hit.clone()
        }));
        assert!(!live.player_interaction(&Event {
            result: 3,
            ..hit.clone()
        }));
        assert!(live.player_interaction(&Event {
            src_agent: 99,
            src_master_instance_id: 10,
            ..hit.clone()
        }));
        assert!(!live.player_interaction(&Event {
            src_agent: 99,
            src_master_instance_id: 11,
            ..hit.clone()
        }));
        live.roster.get_mut(&2).unwrap().team = 0;
        assert!(!live.player_interaction(&hit));
        assert!(live.player_interaction(&Event {
            affinity: 1,
            ..hit.clone()
        }));
        assert!(live.player_interaction(&Event {
            buff: 1,
            value: 0,
            buff_dmg: 25,
            affinity: 1,
            result: 14,
            ..hit
        }));
    }

    #[test]
    fn missing_color_metadata_does_not_discard_live_damage_dps_or_contribution() {
        let mut live = setup();
        live.metadata.clear();
        live.begin();
        let hit = Event {
            time: 1000,
            src_agent: 1,
            dst_agent: 2,
            src_instance_id: 1,
            dst_instance_id: 2,
            value: 100,
            ..empty_event()
        };
        live.push(hit.clone());
        live.push(Event {
            time: 2000,
            value: 200,
            result: 9,
            ..hit
        });
        live.push(Event {
            time: 2100,
            src_agent: 2,
            dst_agent: 1,
            src_instance_id: 2,
            dst_instance_id: 1,
            value: 1,
            skill_id: 717,
            is_buffremove: 1,
            ..empty_event()
        });
        let fight = fight(&live);
        assert_eq!(fight.pov_team, Some(Team::Own));
        assert_eq!(fight.players[0].damage, 300);
        assert!((fight.dps(&fight.players[0]) - 300.0 / 1.1).abs() < 0.001);
        assert_eq!(fight.players[0].strips, 1);
        assert_eq!(fight.players[0].down_contribution, 300);
        assert_eq!(fight.players[1].team, None, "Never assign an enemy color");
        assert_eq!(fight.players[2].team, Some(Team::Own));
        assert_eq!(fight.players[2].damage, 0);
        assert!(fight.players[2].squad_member);
    }

    #[test]
    fn live_uses_hostile_affinity_when_team_ids_are_missing_but_never_counts_npcs() {
        let mut live = setup();
        live.metadata.clear();
        for agent in live.roster.values_mut() {
            agent.team = 0;
        }
        live.begin();
        let hit = Event {
            time: 1000,
            src_agent: 1,
            dst_agent: 2,
            src_instance_id: 1,
            dst_instance_id: 2,
            value: 100,
            affinity: 1,
            ..empty_event()
        };
        live.push(hit.clone());
        live.push(Event {
            time: 2000,
            value: 200,
            result: 9,
            ..hit.clone()
        });
        live.push(Event {
            time: 2100,
            dst_agent: 99,
            value: 10000,
            ..hit.clone()
        });
        live.push(Event {
            time: 2200,
            dst_agent: 3,
            value: 10000,
            affinity: 0,
            ..hit
        });
        let fight = fight(&live);
        assert_eq!(fight.pov_team, Some(Team::Own));
        assert_eq!(fight.players[0].damage, 300);
        assert_eq!(fight.players[0].down_contribution, 300);
        assert_eq!(fight.players[0].live_team_id, None);
        assert_eq!(fight.players[1].team, None);
        assert_eq!(fight.players[2].team, Some(Team::Own));
    }
}
