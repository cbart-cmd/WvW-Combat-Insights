//! Ajustes del usuario en un archivo `clave=valor` en `Documentos\Guild Wars 2\addons\fight_analysis_danger`.

use std::{path::PathBuf, sync::LazyLock};

static DEFAULT_SETTINGS: LazyLock<Settings> = LazyLock::new(|| {
    serde_json::from_str(include_str!("..\\settings.default.json"))
        .expect("settings.default.json must contain valid addon defaults")
});

pub const SUMMARY: usize = 0;
pub const DETAILS: usize = 1;
pub const HEALING: usize = 2;

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowSettings {
    pub enabled: bool,
    pub visible: bool,
    pub auto_show: bool,
    pub hide_in_combat: bool,
    pub fixed_position: bool,
    pub locked: bool,
    pub position: [f32; 2],
    pub width: f32,
    pub height: f32,
    pub columns: u8,
    pub opacity: f32,
    pub follow_latest: bool,
}

impl WindowSettings {
    pub fn for_kind(kind: usize) -> Self {
        DEFAULT_SETTINGS.windows[kind].clone()
    }

    pub fn normalize(&mut self, kind: usize) {
        let defaults = Self::for_kind(kind);
        for (value, default) in self.position.iter_mut().zip(defaults.position) {
            *value = if value.is_finite() {
                value.clamp(0.0, 32000.0)
            } else {
                default
            };
        }
        self.width = if self.width.is_finite() {
            self.width.clamp(125.0, 1000.0)
        } else {
            defaults.width
        };
        self.opacity = if self.opacity.is_finite() {
            self.opacity.clamp(0.0, 1.0)
        } else {
            defaults.opacity
        };
        self.height = if self.height.is_finite() {
            self.height.clamp(0.0, 32000.0)
        } else {
            defaults.height
        };
        self.columns &= if kind == SUMMARY { 0b111 } else { 0b1111 };
        if self.columns == 0 {
            self.columns = defaults.columns;
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    /// Cuantos combates conservar en el historial en memoria.
    pub history_size: usize,
    /// Contar solo el dano y los strips hechos a jugadores de otro color.
    pub vs_players_only: bool,
    /// Logs mas cortos (segundos) no abren la ventana; no retrasa el directo.
    pub min_duration_secs: u32,
    /// Carpeta de logs; vacia = la de arcdps.
    pub log_dir: String,
    pub windows: [WindowSettings; 3],
    /// 0 = total, 1 = healing, 2 = barrier, 3 = cleanses.
    pub healing_sort: usize,
    pub show_weapons: bool,
    /// 0 = first enemy, 1 = own team, 2..4 = red/blue/green.
    pub initial_team: usize,
}

impl Default for Settings {
    fn default() -> Self {
        DEFAULT_SETTINGS.clone()
    }
}

pub const MAX_HISTORY_SIZE: usize = 20;

impl Settings {
    pub fn parse(text: &str) -> Self {
        let mut s = Self::default();
        let own_team_layout =
            text.lines()
                .filter_map(|line| line.split_once('='))
                .any(|(key, value)| {
                    key.trim() == "layout_version"
                        && value.trim().parse::<u32>().is_ok_and(|v| v >= 5)
                });
        let contribution_layout =
            text.lines()
                .filter_map(|line| line.split_once('='))
                .any(|(key, value)| {
                    key.trim() == "layout_version"
                        && value.trim().parse::<u32>().is_ok_and(|v| v >= 4)
                });
        let cleanse_layout =
            text.lines()
                .filter_map(|line| line.split_once('='))
                .any(|(key, value)| {
                    key.trim() == "layout_version"
                        && value.trim().parse::<u32>().is_ok_and(|v| v >= 3)
                });
        let compact_layout =
            text.lines()
                .filter_map(|line| line.split_once('='))
                .any(|(key, value)| {
                    key.trim() == "layout_version"
                        && value.trim().parse::<u32>().is_ok_and(|v| v >= 2)
                });
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            let flag = value == "1" || value.eq_ignore_ascii_case("true");
            match key {
                "history_size" => {
                    s.history_size = value
                        .parse()
                        .unwrap_or(s.history_size)
                        .clamp(1, MAX_HISTORY_SIZE)
                }
                "vs_players_only" => s.vs_players_only = flag,
                "auto_show" => {
                    for window in &mut s.windows {
                        window.auto_show = flag;
                    }
                }
                "hide_in_combat" => {
                    for window in &mut s.windows {
                        window.hide_in_combat = flag;
                    }
                }
                "min_duration_secs" => {
                    s.min_duration_secs = value.parse().unwrap_or(s.min_duration_secs)
                }
                "log_dir" => s.log_dir = value.to_string(),
                "visible" => s.windows[SUMMARY].visible = flag,
                "healing_sort" => s.healing_sort = value.parse().unwrap_or(0).min(3),
                "show_weapons" => s.show_weapons = flag,
                "initial_team" => s.initial_team = value.parse().unwrap_or(0).min(4),
                _ => {}
            }
        }
        // Per-window keys take precedence over legacy settings regardless of line order.
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let Some((kind, field)) = key.trim().split_once('.') else {
                continue;
            };
            let kind = match kind {
                "summary" => SUMMARY,
                "details" => DETAILS,
                "healing" => HEALING,
                _ => continue,
            };
            let w = &mut s.windows[kind];
            let value = value.trim();
            let flag = value == "1" || value.eq_ignore_ascii_case("true");
            match field {
                "enabled" => w.enabled = flag,
                "visible" => w.visible = flag,
                "auto_show" => w.auto_show = flag,
                "hide_in_combat" => w.hide_in_combat = flag,
                "fixed_position" => w.fixed_position = flag,
                "locked" => w.locked = flag,
                "x" => w.position[0] = value.parse().unwrap_or(w.position[0]),
                "y" => w.position[1] = value.parse().unwrap_or(w.position[1]),
                "width" => w.width = value.parse().unwrap_or(w.width),
                "height" => w.height = value.parse().unwrap_or(w.height),
                "columns" => w.columns = value.parse().unwrap_or(w.columns),
                "opacity" => w.opacity = value.parse().unwrap_or(w.opacity),
                "follow_latest" => w.follow_latest = flag,
                _ => {}
            }
        }
        for (kind, window) in s.windows.iter_mut().enumerate() {
            window.normalize(kind);
            if kind == HEALING && !cleanse_layout {
                window.columns |= 8;
            }
            if kind == DETAILS && !contribution_layout {
                window.columns |= 8;
            }
            if !compact_layout && window.width == [380.0, 480.0, 520.0][kind] {
                window.width = 175.0;
            }
        }
        if !own_team_layout && s.initial_team == 0 {
            s.initial_team = 1;
        }
        s
    }

    pub fn to_text(&self) -> String {
        let mut text = format!(
            "layout_version=5\nhistory_size={}\nvs_players_only={}\nmin_duration_secs={}\nlog_dir={}\nhealing_sort={}\nshow_weapons={}\ninitial_team={}\n",
            self.history_size,
            self.vs_players_only as u8,
            self.min_duration_secs,
            self.log_dir,
            self.healing_sort,
            self.show_weapons as u8,
            self.initial_team,
        );
        use std::fmt::Write;
        for (name, w) in ["summary", "details", "healing"]
            .into_iter()
            .zip(&self.windows)
        {
            writeln!(text, "{name}.enabled={}\n{name}.visible={}\n{name}.auto_show={}\n{name}.hide_in_combat={}\n{name}.fixed_position={}\n{name}.locked={}\n{name}.x={}\n{name}.y={}\n{name}.width={}\n{name}.opacity={}\n{name}.follow_latest={}\n{name}.height={}\n{name}.columns={}",
                w.enabled as u8, w.visible as u8, w.auto_show as u8, w.hide_in_combat as u8,
                w.fixed_position as u8, w.locked as u8, w.position[0], w.position[1], w.width, w.opacity,
                w.follow_latest as u8, w.height, w.columns).expect("writing settings to a String");
        }
        text
    }

    pub fn load() -> Self {
        settings_file()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .map(|text| Self::parse(&text))
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let result = (|| {
            let path = settings_file().ok_or_else(|| "Documents folder unavailable".to_string())?;
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            std::fs::write(path, self.to_text()).map_err(|e| e.to_string())
        })();
        if let Err(error) = result {
            nexus::log::log(
                nexus::log::LogLevel::Warning,
                crate::nexusbase::project::NAME,
                format!("Could not save settings: {error}"),
            );
        }
    }
}

/// `Documentos\Guild Wars 2\addons\fight_analysis_danger\settings.txt`.
fn settings_file() -> Option<PathBuf> {
    crate::nexusbase::paths::settings_file()
}

/// Carpeta Documentos del usuario segun Windows (respeta la redireccion a OneDrive).
pub use crate::nexusbase::paths::documents;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_team_defaults_to_own_and_migrates_the_old_enemy_default_once() {
        assert_eq!(Settings::default().initial_team, 1);
        assert_eq!(
            Settings::parse("layout_version=4\ninitial_team=0").initial_team,
            1
        );
        assert_eq!(
            Settings::parse("layout_version=4\ninitial_team=3").initial_team,
            3
        );
        let mut customized = Settings::default();
        customized.initial_team = 0;
        assert_eq!(Settings::parse(&customized.to_text()).initial_team, 0);
    }

    #[test]
    fn cleanse_column_migrates_once_and_remains_optional() {
        let old = Settings::parse("layout_version=2\nhealing.columns=6\nhealing_sort=3");
        assert_eq!(old.windows[HEALING].columns, 14);
        assert_eq!(old.healing_sort, 3);
        assert_eq!(
            Settings::parse("healing.columns=0").windows[HEALING].columns,
            14
        );
        let mut customized = old;
        customized.windows[HEALING].columns = 6;
        assert_eq!(Settings::parse(&customized.to_text()), customized);
        assert_eq!(
            Settings::parse("layout_version=3\nhealing.columns=8").windows[HEALING].columns,
            8
        );
    }

    #[test]
    fn down_contribution_column_migrates_and_obsolete_settings_are_dropped() {
        let old = Settings::parse("layout_version=3\ndetails.columns=6");
        assert_eq!(old.windows[DETAILS].columns, 14);
        let mut customized = old;
        customized.windows[DETAILS].columns = 6;
        assert_eq!(Settings::parse(&customized.to_text()), customized);
        let legacy = Settings::parse(
            "top=4\ntop_dps=2\ntop_strips=3\ndown_contribution_secs=5\nshow_weapons=0",
        );
        assert!(!legacy.show_weapons);
        let text = legacy.to_text();
        for obsolete in ["top=", "top_dps=", "top_strips=", "down_contribution_secs="] {
            assert!(!text.contains(obsolete));
        }
        assert_eq!(
            Settings::parse("layout_version=4\ndetails.columns=8").windows[DETAILS].columns,
            8
        );
    }

    #[test]
    fn weapon_visibility_defaults_on_and_preserves_saved_choice() {
        assert!(Settings::default().show_weapons);
        assert!(Settings::parse("").show_weapons);
        let hidden = Settings::parse("show_weapons=0");
        assert!(!hidden.show_weapons);
        assert_eq!(Settings::parse(&hidden.to_text()), hidden);
        assert!(Settings::parse("show_weapons=1").show_weapons);
    }

    #[test]
    fn round_trip() {
        let s = Settings {
            vs_players_only: false,
            log_dir: r"D:\logs".into(),
            ..Default::default()
        };
        let mut s = s;
        s.windows[HEALING].enabled = true;
        s.windows[HEALING].visible = true;
        s.windows[DETAILS].position = [420.0, 180.0];
        s.windows[DETAILS].height = 235.0;
        s.windows[DETAILS].columns = 7;
        s.windows[HEALING].columns = 2;
        assert_eq!(Settings::parse(&s.to_text()), s);
        assert_eq!(
            Settings::parse("history_size=99").history_size,
            MAX_HISTORY_SIZE
        );
        assert_eq!(Settings::parse("history_size=0").history_size, 1);
        assert_eq!(Settings::parse("basura"), Settings::default());
        assert_eq!(
            Settings::parse("details.columns=0").windows[DETAILS].columns,
            14
        );
        assert_eq!(
            Settings::parse("healing.columns=255").windows[HEALING].columns,
            15
        );
        assert_eq!(
            Settings::parse("summary.height=NaN").windows[SUMMARY].height,
            78.0
        );
    }

    #[test]
    fn migrates_legacy_visibility_and_normalizes_window_geometry() {
        let s = Settings::parse(
            "visible=1\nauto_show=0\nhide_in_combat=0\nhealing_rows=99\nsummary.width=999\nhealing.x=NaN\ndetails.width=inf\nsummary.auto_show=1",
        );
        assert!(s.windows[SUMMARY].visible);
        assert!(s.windows[SUMMARY].auto_show);
        assert!(!s.windows[HEALING].auto_show);
        assert!(!s.windows[DETAILS].hide_in_combat);
        assert_eq!(s.windows[SUMMARY].width, 999.0);
        assert_eq!(
            s.windows[HEALING].position,
            WindowSettings::for_kind(HEALING).position
        );
        assert_eq!(s.windows[DETAILS].width, 300.0);
        assert_eq!(Settings::parse("healing_rows=0"), Settings::default());
        assert!(!s.to_text().contains("healing_rows"));
        assert!(Settings::parse("summary.auto_show=1\nauto_show=0").windows[SUMMARY].auto_show);
    }

    #[test]
    fn compact_defaults_and_opacity_preserve_explicit_user_sizes() {
        let s = Settings::parse(
            "layout_version=2\nsummary.width=380\ndetails.width=100\nhealing.width=175\nsummary.opacity=NaN\ndetails.opacity=2\nhealing.opacity=-1\nshow_healing_bars=1",
        );
        assert_eq!(s.windows[SUMMARY].width, 380.0);
        assert_eq!(s.windows[DETAILS].width, 125.0);
        assert_eq!(s.windows[HEALING].width, 175.0);
        assert_eq!(s.windows[SUMMARY].opacity, 0.49);
        assert_eq!(s.windows[DETAILS].opacity, 1.0);
        assert_eq!(s.windows[HEALING].opacity, 0.0);
        assert_eq!(Settings::parse(&s.to_text()), s);
        assert!(!s.to_text().contains("show_healing_bars"));
        let migrated = Settings::parse("summary.width=380\ndetails.width=480\nhealing.width=520");
        assert!(migrated.windows.iter().all(|w| w.width == 175.0));
        assert_eq!(
            Settings::parse("summary.width=410").windows[SUMMARY].width,
            410.0
        );
    }

    #[test]
    fn new_installation_defaults_match_selected_preferences_without_position_locks() {
        let defaults = Settings::default();
        assert_eq!(Settings::parse(""), defaults);
        assert_eq!(Settings::parse(&defaults.to_text()), defaults);
        for (kind, window) in defaults.windows.iter().enumerate() {
            assert!(window.enabled && window.visible && window.auto_show && !window.hide_in_combat);
            assert!(!window.locked && !window.fixed_position && !window.follow_latest);
            assert_eq!(
                window.position,
                [[391.0, 1.0], [233.0, 624.0], [0.0, 624.0]][kind]
            );
            assert_eq!(window.width, [259.0, 300.0, 234.0][kind]);
            assert_eq!(window.opacity, [0.49, 0.38, 0.32][kind]);
            assert_eq!(window.height, [78.0, 112.0, 116.0][kind]);
            assert_eq!(window.columns, if kind == SUMMARY { 6 } else { 14 });
        }
        let saved = Settings::parse(
            "layout_version=2\nsummary.enabled=0\nsummary.visible=0\nsummary.width=175\nsummary.opacity=0.75\nsummary.x=50\nsummary.y=60\nsummary.locked=1\nsummary.fixed_position=1\nsummary.follow_latest=1\ndetails.height=234",
        );
        assert!(!saved.windows[SUMMARY].enabled && !saved.windows[SUMMARY].visible);
        assert!(
            saved.windows[SUMMARY].locked
                && saved.windows[SUMMARY].fixed_position
                && saved.windows[SUMMARY].follow_latest
        );
        assert_eq!(saved.windows[SUMMARY].width, 175.0);
        assert_eq!(saved.windows[SUMMARY].opacity, 0.75);
        assert_eq!(saved.windows[SUMMARY].position, [50.0, 60.0]);
        assert_eq!(saved.windows[DETAILS].height, 234.0);
    }

    #[test]
    fn embedded_json_is_complete_and_within_supported_limits() {
        let defaults: Settings = serde_json::from_str(include_str!("..\\settings.default.json"))
            .expect("valid complete JSON defaults");
        assert_eq!(defaults, Settings::default());
        assert_eq!(Settings::parse(&defaults.to_text()), defaults);
        for (kind, window) in defaults.windows.iter().enumerate() {
            let mut normalized = window.clone();
            normalized.normalize(kind);
            assert_eq!(&normalized, window);
            assert_eq!(WindowSettings::for_kind(kind), *window);
        }
        assert!(serde_json::from_str::<Settings>("{}").is_err());
        let invalid = include_str!("..\\settings.default.json")
            .replace("\"history_size\"", "\"typo_history_size\"");
        assert!(serde_json::from_str::<Settings>(&invalid).is_err());
    }
}
