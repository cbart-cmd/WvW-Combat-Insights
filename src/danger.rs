//! Ranking de los enemigos mas peligrosos de cada color.

use crate::evtc::{Fight, Team};
use crate::i18n::{Language, Text};

pub fn highlight_score(player: &crate::evtc::Player) -> u128 {
    u128::from(player.damage)
        + u128::from(player.strips) * 2000
        + u128::from(player.down_contribution)
}

pub fn highlight_rows(fight: &Fight, team: Team, sort: usize, ascending: bool) -> Vec<usize> {
    let mut rows: Vec<_> = (0..fight.players.len())
        .filter(|&i| {
            let player = &fight.players[i];
            player.team == Some(team) && (player.damage > 0 || player.strips > 0)
        })
        .collect();
    rows.sort_by(|&a, &b| {
        let value = |i: usize| match sort {
            0 | 1 => u128::from(fight.players[i].damage),
            2 => u128::from(fight.players[i].strips),
            3 => u128::from(fight.players[i].down_contribution),
            _ => highlight_score(&fight.players[i]),
        };
        let order = if ascending {
            value(a).cmp(&value(b))
        } else {
            value(b).cmp(&value(a))
        };
        order.then(a.cmp(&b))
    });
    rows
}

/// Los peligrosos de un color enemigo (indices en `Fight::players`).
#[derive(Clone, Debug)]
pub struct TeamDanger {
    pub team: Team,
    /// Jugadores de ese color con alguna actividad en el log.
    pub active: usize,
    pub top_dps: Vec<usize>,
    pub top_strips: Vec<usize>,
}

impl TeamDanger {
    pub fn rows(&self) -> Vec<(String, usize)> {
        self.rows_in(Language::English)
    }

    pub fn rows_in(&self, language: Language) -> Vec<(String, usize)> {
        let mut rows: Vec<_> = self
            .top_dps
            .iter()
            .enumerate()
            .map(|(pos, &index)| (format!("DPS {}", pos + 1), index))
            .collect();
        for (pos, &index) in self.top_strips.iter().enumerate() {
            let label = language.format(Text::StripPosition, &[(pos + 1).to_string()]);
            if let Some((existing, _)) = rows.iter_mut().find(|(_, player)| *player == index) {
                existing.push_str(&format!(" / {label}"));
            } else {
                rows.push((label, index));
            }
        }
        rows
    }
}

/// Top `n` por DPS y por strips de cada color enemigo. Si no se sabe el color propio se listan los tres.
pub fn rank(fight: &Fight, n: usize) -> Vec<TeamDanger> {
    rank_categories(fight, n, n)
}

pub fn rank_categories(fight: &Fight, top_dps: usize, top_strips: usize) -> Vec<TeamDanger> {
    Team::ALL
        .into_iter()
        .filter(|&team| Some(team) != fight.pov_team)
        .filter_map(|team| rank_team(fight, team, top_dps, top_strips))
        .collect()
}

/// Same ranking criteria for a selected color, including the player's own team.
pub fn rank_team(
    fight: &Fight,
    team: Team,
    top_dps: usize,
    top_strips: usize,
) -> Option<TeamDanger> {
    let members: Vec<usize> = (0..fight.players.len())
        .filter(|&i| {
            let p = &fight.players[i];
            p.team == Some(team) && (p.damage > 0 || p.strips > 0)
        })
        .collect();
    if members.is_empty() {
        return None;
    }
    let mut by_dps: Vec<usize> = members
        .iter()
        .copied()
        .filter(|&i| fight.players[i].damage > 0)
        .collect();
    by_dps.sort_by(|&a, &b| fight.players[b].damage.cmp(&fight.players[a].damage));
    by_dps.truncate(top_dps);
    let mut by_strips: Vec<usize> = members
        .iter()
        .copied()
        .filter(|&i| fight.players[i].strips > 0)
        .collect();
    by_strips.sort_by(|&a, &b| {
        let (pa, pb) = (&fight.players[a], &fight.players[b]);
        pb.strips.cmp(&pa.strips).then(pb.damage.cmp(&pa.damage))
    });
    by_strips.truncate(top_strips);
    Some(TeamDanger {
        team,
        active: members.len(),
        top_dps: by_dps,
        top_strips: by_strips,
    })
}

/// 12345 -> "12.3k", 1234567 -> "1.23M".
pub fn short_number(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("{:.2}M", value / 1_000_000.0)
    } else if value >= 10_000.0 {
        format!("{:.1}k", value / 1_000.0)
    } else if value >= 1_000.0 {
        format!("{:.2}k", value / 1_000.0)
    } else {
        format!("{value:.0}")
    }
}

/// Resumen en texto plano (para copiar al chat o al portapapeles).
pub fn summary(fight: &Fight, ranks: &[TeamDanger]) -> String {
    summary_in(fight, ranks, fight.language.unwrap_or(Language::Spanish))
}

/// Resumen en una linea por color, corto para pegarlo en el chat del juego.
pub fn chat_lines(fight: &Fight, ranks: &[TeamDanger]) -> Vec<String> {
    chat_lines_in(fight, ranks, fight.language.unwrap_or(Language::Spanish))
}

/// Resumen compacto para pegar en el chat, nombrando cada color una sola vez.
pub fn chat_summary(fight: &Fight, ranks: &[TeamDanger]) -> String {
    chat_lines(fight, ranks).join("\n")
}

pub fn summary_in(fight: &Fight, ranks: &[TeamDanger], language: Language) -> String {
    let mut out = String::new();
    for danger in ranks {
        let color = language.team(danger.team).to_uppercase();
        for (label, i) in danger.rows_in(language) {
            let p = &fight.players[i];
            out.push_str(&language.format(
                Text::SummaryRow,
                &[
                    color.clone(),
                    label,
                    language.spec(p.profession, p.elite).into(),
                    short_number(p.damage as f64),
                    p.strips.to_string(),
                ],
            ));
        }
    }
    out
}

pub fn chat_summary_in(fight: &Fight, ranks: &[TeamDanger], language: Language) -> String {
    chat_lines_in(fight, ranks, language).join("\n")
}

fn chat_lines_in(fight: &Fight, ranks: &[TeamDanger], language: Language) -> Vec<String> {
    ranks
        .iter()
        .map(|danger| {
            let list = danger
                .rows_in(language)
                .into_iter()
                .map(|(_, i)| {
                    let p = &fight.players[i];
                    language.format(
                        Text::ChatPlayer,
                        &[
                            language.spec(p.profession, p.elite).into(),
                            short_number(p.damage as f64),
                            p.strips.to_string(),
                        ],
                    )
                })
                .collect::<Vec<_>>()
                .join(" | ");
            format!("{}: {list}", language.team(danger.team).to_uppercase())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evtc::Player;

    fn player(team: Team, damage: u64, strips: u32) -> Player {
        Player {
            team: Some(team),
            damage,
            strips,
            profession: 1,
            elite: 27,
            ..Default::default()
        }
    }

    #[test]
    fn ranks_enemy_colors_only() {
        let fight = Fight {
            duration_ms: 10_000,
            pov_team: Some(Team::Green),
            pov_subgroup: None,
            end_unix: 0,
            language: None,
            players: vec![
                player(Team::Red, 100, 0),
                player(Team::Red, 300, 5),
                player(Team::Red, 200, 9),
                player(Team::Blue, 50, 1),
                player(Team::Green, 9999, 99),
            ],
            ..Default::default()
        };
        let ranks = rank(&fight, 2);
        assert_eq!(ranks.len(), 2);
        assert_eq!(ranks[0].team, Team::Red);
        assert_eq!(ranks[0].top_dps, vec![1, 2]);
        assert_eq!(ranks[0].top_strips, vec![2, 1]);
        assert_eq!(ranks[1].team, Team::Blue);
        assert_eq!(fight.dps(&fight.players[1]), 30.0);
        assert_eq!(
            chat_lines(&fight, &ranks)[0],
            "ROJO: Cazadragones 300 dps, 5 strips | Cazadragones 200 dps, 9 strips"
        );
        assert_eq!(
            chat_summary(&fight, &ranks),
            "ROJO: Cazadragones 300 dps, 5 strips | Cazadragones 200 dps, 9 strips\nAZUL: Cazadragones 50 dps, 1 strips"
        );
    }

    #[test]
    fn overlapping_tops_show_three_unique_players_without_backfilling() {
        let fight = Fight {
            duration_ms: 10_000,
            players: vec![
                player(Team::Red, 400, 20),
                player(Team::Red, 300, 1),
                player(Team::Red, 200, 10),
                player(Team::Red, 100, 5),
            ],
            ..Default::default()
        };
        let ranks = rank(&fight, 2);
        assert_eq!(
            ranks[0].rows(),
            vec![
                ("DPS 1 / Strip 1".into(), 0),
                ("DPS 2".into(), 1),
                ("Strip 2".into(), 2),
            ]
        );
        assert_eq!(summary(&fight, &ranks).lines().count(), 3);
        assert_eq!(
            chat_lines(&fight, &ranks)[0]
                .matches("Cazadragones")
                .count(),
            3
        );
        let independent = rank_categories(&fight, 1, 3);
        assert_eq!(independent[0].top_dps, vec![0]);
        assert_eq!(independent[0].top_strips, vec![0, 2, 3]);
        assert_eq!(independent[0].rows().len(), 3);
    }

    #[test]
    fn rows_preserve_positions_with_full_overlap_and_disjoint_tops() {
        let mut danger = TeamDanger {
            team: Team::Red,
            active: 4,
            top_dps: vec![0, 1],
            top_strips: vec![1, 0],
        };
        assert_eq!(
            danger.rows(),
            vec![("DPS 1 / Strip 2".into(), 0), ("DPS 2 / Strip 1".into(), 1),]
        );
        danger.top_strips = vec![2, 3];
        assert_eq!(danger.rows().len(), 4);
        danger.top_dps.clear();
        assert_eq!(
            danger.rows(),
            vec![("Strip 1".into(), 2), ("Strip 2".into(), 3)]
        );
    }

    #[test]
    fn localized_exports_preserve_ranking_overlap_and_all_colors() {
        let fight = Fight {
            duration_ms: 10_000,
            pov_team: Some(Team::Green),
            players: vec![
                player(Team::Red, 400, 20),
                player(Team::Red, 300, 1),
                player(Team::Red, 200, 10),
                player(Team::Blue, 100, 1),
            ],
            ..Default::default()
        };
        let ranks = rank(&fight, 2);
        for (language, red, blue, spec, strips) in [
            (Language::Spanish, "ROJO", "AZUL", "Cazadragones", "strips"),
            (Language::English, "RED", "BLUE", "Dragonhunter", "strips"),
            (
                Language::French,
                "ROUGE",
                "BLEU",
                "Draconnier",
                "dissipations",
            ),
            (
                Language::German,
                "ROT",
                "BLAU",
                "Drachenjäger",
                "Entfernungen",
            ),
        ] {
            let summary = summary_in(&fight, &ranks, language);
            let chat = chat_summary_in(&fight, &ranks, language);
            assert_eq!(summary.lines().count(), 4);
            assert_eq!(chat.lines().count(), 2);
            assert_eq!(chat.matches(spec).count(), 4);
            assert!(chat.starts_with(&format!("{red}:")));
            assert!(
                chat.lines()
                    .nth(1)
                    .unwrap()
                    .starts_with(&format!("{blue}:"))
            );
            assert!(chat.contains(&format!("20 {strips}")));
            assert!(!summary.contains("{0}"));
            assert_eq!(ranks[0].rows_in(language).len(), 3);
            let localized_fight = Fight {
                language: Some(language),
                ..fight.clone()
            };
            assert_eq!(super::summary(&localized_fight, &ranks), summary);
            assert_eq!(chat_summary(&localized_fight, &ranks), chat);
        }
    }

    #[test]
    fn selected_team_can_rank_allies_without_changing_enemy_exports() {
        let fight = Fight {
            pov_team: Some(Team::Green),
            players: vec![player(Team::Green, 800, 9), player(Team::Red, 500, 3)],
            ..Default::default()
        };
        assert_eq!(rank(&fight, 2).len(), 1);
        assert_eq!(rank(&fight, 2)[0].team, Team::Red);
        let allies = rank_team(&fight, Team::Green, 2, 2).unwrap();
        assert_eq!(allies.rows().len(), 1);
        assert_eq!(allies.top_dps, vec![0]);
        assert_eq!(allies.top_strips, vec![0]);
        assert!(rank_team(&fight, Team::Blue, 2, 2).is_none());
    }

    #[test]
    fn short_numbers() {
        assert_eq!(short_number(950.0), "950");
        assert_eq!(short_number(1234.0), "1.23k");
        assert_eq!(short_number(12345.0), "12.3k");
        assert_eq!(short_number(1_234_567.0), "1.23M");
    }
}
