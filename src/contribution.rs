//! Native AxiLog 1.16.0 damage-to-downs methodology; see calculos.md.

use super::{Event, Player, RESULT_DOWNED, sc};
use std::collections::HashMap;

struct Instances(HashMap<u16, Vec<(u64, u64)>>);

impl Instances {
    fn build(events: &[Event]) -> Self {
        let mut histories: HashMap<u16, Vec<(u64, u64)>> = HashMap::new();
        for event in events {
            if matches!(event.is_statechange, sc::EXTENSION | sc::EXTENSION_COMBAT) {
                continue;
            }
            for (instance, address) in [
                (event.src_instid, event.src_agent),
                (event.dst_instid, event.dst_agent),
            ] {
                if instance == 0 || address == 0 {
                    continue;
                }
                let history = histories.entry(instance).or_default();
                if history
                    .last()
                    .is_some_and(|&(_, previous)| previous == address)
                {
                    continue;
                }
                let position = history.partition_point(|&(time, _)| time <= event.time);
                history.insert(position, (event.time, address));
            }
        }
        Self(histories)
    }

    fn at(&self, instance: u16, time: u64) -> Option<u64> {
        let history = self.0.get(&instance)?;
        let end = history.partition_point(|&(registered, _)| registered <= time);
        end.checked_sub(1).map(|index| history[index].1)
    }

    fn contributor(&self, event: &Event) -> Option<u64> {
        if event.src_agent == 0
            || (event.src_instid != 0
                && self.at(event.src_instid, event.time) != Some(event.src_agent))
        {
            return None;
        }
        Some(
            self.at(event.src_master_instid, event.time)
                .filter(|_| event.src_master_instid != 0)
                .unwrap_or(event.src_agent),
        )
    }
}

pub(super) fn collect(events: &[Event], by_addr: &HashMap<u64, usize>, players: &mut [Player]) {
    let mut downs: Vec<_> = events
        .iter()
        .filter(|event| event.is_statechange == sc::NONE && event.result == RESULT_DOWNED)
        .filter_map(|event| {
            by_addr
                .get(&event.dst_agent)
                .map(|&target| (event.time, target))
        })
        .collect();
    if downs.is_empty() {
        return;
    }
    downs.sort_by_key(|&(time, _)| time);
    let instances = Instances::build(events);
    let log_start = events.first().map_or(0, |event| event.time);
    let mut health = vec![Vec::new(); players.len()];
    let mut damage = vec![Vec::new(); players.len()];
    for event in events {
        if event.is_statechange == sc::HEALTH_UPDATE {
            if (9900..=20000).contains(&event.dst_agent)
                && let Some(&target) = by_addr.get(&event.src_agent)
            {
                health[target].push(event.time);
            }
        } else if event.is_statechange == sc::NONE
            && event.is_activation == 0
            && event.is_buffremove == 0
            && !matches!(event.result, 10 | 12)
            && let Some(&target) = by_addr.get(&event.dst_agent)
        {
            let amount = if event.buff == 1 {
                event.buff_dmg
            } else {
                event.value
            };
            if amount > 0
                && let Some(actor) = instances
                    .contributor(event)
                    .and_then(|address| by_addr.get(&address).copied())
                && players[actor].opposed(&players[target], event.affinity)
            {
                damage[target].push((event.time, actor, amount as u64));
            }
        }
    }
    for times in &mut health {
        times.sort_unstable();
    }
    for hits in &mut damage {
        hits.sort_by_key(|&(time, _, _)| time);
    }
    let mut reset = vec![0; players.len()];
    for (down, target) in downs {
        let anchors = &health[target];
        let end = anchors.partition_point(|&time| time <= down);
        let anchor = end.checked_sub(1).map_or(log_start, |index| anchors[index]);
        let start = anchor
            .saturating_sub(2000)
            .max(log_start)
            .max(reset[target]);
        reset[target] = down.saturating_add(2100);
        if start > down {
            continue;
        }
        let hits = &damage[target];
        let first = hits.partition_point(|&(time, _, _)| time < start);
        let last = hits.partition_point(|&(time, _, _)| time <= down);
        for &(_, actor, amount) in &hits[first..last] {
            players[actor].down_contribution =
                players[actor].down_contribution.saturating_add(amount);
        }
    }
}
