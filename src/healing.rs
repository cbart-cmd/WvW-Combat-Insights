//! Healing Stats wire protocol, revisions 1/2. Sources:
//! https://github.com/Krappa322/arcdps_healing_stats/blob/master/src/AddonVersion.h
//! https://github.com/Krappa322/arcdps_healing_stats/blob/master/src/EventProcessor.cpp
//! https://github.com/Krappa322/arcdps_healing_stats/blob/master/src/AggregatedStats.cpp
//! https://github.com/baaron4/GW2-Elite-Insights-Parser/blob/master/GW2EI.Library/GW2EI.Services/GW2EIEvtcParser/Extensions/ExtensionHelper.cs
//! EI prefers source-peer observations over destination-peer observations per source.
//! Peer flags establish participation, not recording continuity or absence of heals.

use super::{Event, HealingCoverage, Player, is_condition, sc};
use std::collections::{HashMap, HashSet};

const SIGNATURE: u32 = 0x9c9b3c99;

fn member(players: &[Player], groups: &[Option<u8>], pov: Option<usize>, i: usize) -> bool {
    let Some(pov) = pov else { return false };
    !players[pov].account.is_empty()
        && groups[pov].is_some_and(|group| (1..=15).contains(&group))
        && !players[i].account.is_empty()
        && groups[i].is_some_and(|group| (1..=15).contains(&group))
        && !matches!((players[pov].team, players[i].team), (Some(a), Some(b)) if a != b)
}

pub(super) fn collect(
    events: &[Event],
    by_addr: &HashMap<u64, usize>,
    pov_addr: Option<u64>,
    initial_subgroups: &[Option<u8>],
    players: &mut [Player],
) -> HealingCoverage {
    let pov = pov_addr.and_then(|addr| by_addr.get(&addr).copied());
    let final_groups: Vec<_> = players.iter().map(|p| p.subgroup).collect();
    for i in 0..players.len() {
        players[i].squad_member = member(players, &final_groups, pov, i);
    }
    let mut groups = initial_subgroups.to_vec();
    let mut coverage = HealingCoverage::Missing;
    let mut enabled = false;
    // Extension addresses from network peers are not trustworthy. Resolve their
    // instance IDs only from ordinary EVTC events, including recorded pet masters.
    let mut instances: HashMap<u16, Option<usize>> = HashMap::new();
    let mut observations = Vec::new();
    for ev in events {
        if ev.is_statechange == sc::EXTENSION && ev.pad == 0 {
            if ev.src_agent as u32 == SIGNATURE {
                let revision = (ev.src_agent >> 32) as u32 & 0x00ff_ffff;
                enabled = matches!(revision, 1 | 2);
                if enabled {
                    coverage = HealingCoverage::Partial;
                } else if coverage != HealingCoverage::Partial {
                    coverage = HealingCoverage::Unsupported;
                }
            }
            continue;
        }
        if matches!(ev.is_statechange, sc::EXTENSION | sc::EXTENSION_COMBAT) {
            if ev.pad != SIGNATURE {
                continue;
            }
            if !enabled {
                if coverage == HealingCoverage::Missing {
                    coverage = HealingCoverage::Unsupported;
                }
                continue;
            }
            if ev.is_activation != 0 || ev.is_buffremove != 0 {
                continue;
            }
            let amount = match ev.buff {
                0 if ev.value < 0 => -(ev.value as i64) as u64,
                1.. if ev.value == 0 && ev.buff_dmg < 0 => -(ev.buff_dmg as i64) as u64,
                _ => continue,
            };
            let resolve = |instance: u16, master: u16| {
                instances
                    .get(&instance)
                    .copied()
                    .flatten()
                    .or_else(|| instances.get(&master).copied().flatten())
            };
            let (Some(src), Some(dst)) = (
                resolve(ev.src_instid, ev.src_master_instid),
                resolve(ev.dst_instid, ev.dst_master_instid),
            ) else {
                continue;
            };
            if !member(players, &groups, pov, src) || !member(players, &groups, pov, dst) {
                continue;
            }
            // Legacy revision-1 records lack peer bits and are source observations.
            let source_peer = ev.is_offcycle & 0x80 != 0 || ev.is_offcycle & 0xc0 == 0;
            observations.push((src, dst, amount, ev.is_shields != 0, source_peer, *ev));
            continue;
        }
        if ev.is_statechange == sc::ENTER_COMBAT
            && let Some(&i) = by_addr.get(&ev.src_agent)
        {
            groups[i] = u8::try_from(ev.dst_agent).ok().filter(|group| *group <= 15);
        }
        if ev.is_statechange == sc::DESPAWN {
            instances.remove(&ev.src_instid);
            continue;
        }
        // Only event kinds whose src/dst are actual agents can seed instance IDs.
        if !matches!(
            ev.is_statechange,
            sc::NONE | sc::ENTER_COMBAT | 2
                ..=8 | sc::POINT_OF_VIEW | sc::TEAM_CHANGE | sc::BUFF_REMOVE_ALL
        ) {
            continue;
        }
        for (side, (addr, instance, master)) in [
            (ev.src_agent, ev.src_instid, ev.src_master_instid),
            (ev.dst_agent, ev.dst_instid, ev.dst_master_instid),
        ]
        .into_iter()
        .enumerate()
        {
            if addr == 0 || instance == 0 {
                continue;
            }
            // ENTER_COMBAT dst_agent is a subgroup, not an agent address.
            if ev.is_statechange != sc::NONE
                && ev.is_statechange != sc::BUFF_REMOVE_ALL
                && side == 1
            {
                continue;
            }
            let owner = by_addr
                .get(&addr)
                .copied()
                .or_else(|| instances.get(&master).copied().flatten());
            instances.insert(instance, owner);
        }
        // Like strips, count one full removal per condition, not its stacks.
        // The removal source loses the condition; its destination cleanses it.
        if ((ev.is_statechange == sc::BUFF_REMOVE_ALL && matches!(ev.is_buffremove, 0 | 1))
            || (ev.is_statechange == sc::NONE && ev.is_buffremove == 1))
            && ev.is_activation == 0
            && ev.value > 0
            && is_condition(ev.skill_id)
        {
            let resolve = |addr, master| {
                by_addr
                    .get(&addr)
                    .copied()
                    .or_else(|| instances.get(&master).copied().flatten())
            };
            if let (Some(recipient), Some(cleanser)) = (
                resolve(ev.src_agent, ev.src_master_instid),
                resolve(ev.dst_agent, ev.dst_master_instid),
            ) && member(players, &groups, pov, recipient)
                && member(players, &groups, pov, cleanser)
            {
                players[cleanser].cleanses += 1;
            }
        }
    }
    let source_peers: HashSet<_> = observations
        .iter()
        .filter(|(_, _, _, _, peer, _)| *peer)
        .map(|(src, _, _, barrier, _, ev)| (*src, ev.src_instid, *barrier))
        .collect();
    let mut seen = HashSet::new();
    for (src, dst, amount, barrier, source_peer, ev) in observations {
        if !source_peer && source_peers.contains(&(src, ev.src_instid, barrier)) {
            continue;
        }
        // Collapse identical peer echoes, but preserve different skills/recipients
        // and downed-state flags. Same-millisecond identical genuine ticks cannot
        // be distinguished by this wire format: these totals are lower bounds.
        if !seen.insert((
            src,
            dst,
            ev.src_instid,
            ev.dst_instid,
            ev.time,
            ev.skill_id,
            amount,
            barrier,
            ev.buff,
            ev.is_offcycle & 0x3f,
        )) {
            continue;
        }
        if barrier {
            players[src].barrier_coverage = HealingCoverage::Partial;
            players[src].barrier += amount;
        } else {
            players[src].healing_coverage = HealingCoverage::Partial;
            players[src].healing += amount;
        }
    }
    if coverage == HealingCoverage::Unsupported {
        for player in players.iter_mut().filter(|p| p.squad_member) {
            player.healing_coverage = HealingCoverage::Unsupported;
            player.barrier_coverage = HealingCoverage::Unsupported;
        }
    }
    if coverage == HealingCoverage::Partial
        && !pov.is_some_and(|i| member(players, &final_groups, pov, i))
    {
        for player in players.iter_mut().filter(|p| {
            p.healing_coverage == HealingCoverage::Missing
                && p.barrier_coverage == HealingCoverage::Missing
        }) {
            player.healing_coverage = HealingCoverage::UnknownGroup;
            player.barrier_coverage = HealingCoverage::UnknownGroup;
        }
        return HealingCoverage::UnknownGroup;
    }
    coverage
}
