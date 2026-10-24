//! Localiza la carpeta de logs de arcdps y los `.zevtc` dentro de ella (incluidas subcarpetas).

use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

/// Carpeta de logs: la elegida por el usuario, la `boss_encounter_path` de `arcdps.ini` o la de por defecto.
pub fn log_dir(custom: &str, arcdps_ini: Option<PathBuf>) -> Option<PathBuf> {
    if !custom.trim().is_empty() {
        return Some(PathBuf::from(custom.trim()));
    }
    if let Some(dir) = arcdps_ini
        .and_then(|ini| std::fs::read(ini).ok())
        .and_then(|raw| ini_log_path(&String::from_utf8_lossy(&raw)))
    {
        return Some(dir);
    }
    crate::settings::documents().map(|d| {
        d.join("Guild Wars 2")
            .join("addons")
            .join("arcdps")
            .join("arcdps.cbtlogs")
    })
}

/// `boss_encounter_path` de la seccion `[session]` de `arcdps.ini`.
fn ini_log_path(ini: &str) -> Option<PathBuf> {
    let mut in_session = false;
    for line in ini.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_session = line.eq_ignore_ascii_case("[session]");
        } else if in_session
            && let Some((key, value)) = line.split_once('=')
            && key.trim().eq_ignore_ascii_case("boss_encounter_path")
            && !value.trim().is_empty()
        {
            return Some(PathBuf::from(value.trim()));
        }
    }
    None
}

fn is_log(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("zevtc") || e.eq_ignore_ascii_case("evtc"))
}

/// Recorre `dir` (con subcarpetas, hasta 4 niveles) llamando a `visit` con cada log y su fecha de modificacion.
fn walk(dir: &Path, depth: u32, visit: &mut dyn FnMut(&Path, SystemTime, u64)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        let path = entry.path();
        if meta.is_dir() {
            if depth < 4 {
                walk(&path, depth + 1, visit);
            }
        } else if is_log(&path)
            && let Ok(modified) = meta.modified()
        {
            visit(&path, modified, meta.len());
        }
    }
}

/// Log mas reciente modificado despues de `since`, con su tamano.
pub fn newest(dir: &Path, since: SystemTime) -> Option<(PathBuf, u64)> {
    let mut best: Option<(PathBuf, SystemTime, u64)> = None;
    walk(dir, 0, &mut |path, modified, size| {
        if modified > since && best.as_ref().is_none_or(|(_, t, _)| modified > *t) {
            best = Some((path.to_path_buf(), modified, size));
        }
    });
    best.map(|(path, _, size)| (path, size))
}

/// Logs mas recientes por fecha de modificacion, del mas nuevo al mas antiguo.
pub fn recent(dir: &Path, limit: usize) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk(dir, 0, &mut |path, modified, _| {
        files.push((path.to_path_buf(), modified))
    });
    sort_recent(files, limit)
}

fn sort_recent(mut files: Vec<(PathBuf, SystemTime)>, limit: usize) -> Vec<PathBuf> {
    files.sort_unstable_by(|(path_a, time_a), (path_b, time_b)| {
        time_b.cmp(time_a).then_with(|| path_a.cmp(path_b))
    });
    files
        .into_iter()
        .take(limit)
        .map(|(path, _)| path)
        .collect()
}

/// Log con ese nombre de archivo (el mas reciente si hay varios).
pub fn find(dir: &Path, file_name: &str) -> Option<PathBuf> {
    let mut best: Option<(PathBuf, SystemTime)> = None;
    walk(dir, 0, &mut |path, modified, _| {
        let same = path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case(file_name));
        if same && best.as_ref().is_none_or(|(_, t)| modified > *t) {
            best = Some((path.to_path_buf(), modified));
        }
    });
    best.map(|(path, _)| path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_recent_orders_logs_and_obeys_limit() {
        let files: Vec<_> = (0..12)
            .map(|i| {
                (
                    PathBuf::from(format!("{i:02}.zevtc")),
                    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(i),
                )
            })
            .collect();
        let recent = sort_recent(files, 10);
        assert_eq!(recent.len(), 10);
        assert_eq!(recent[0], PathBuf::from("11.zevtc"));
        assert_eq!(recent[9], PathBuf::from("02.zevtc"));
    }

    #[test]
    fn reads_boss_encounter_path() {
        let ini =
            "[other]\nboss_encounter_path=X\n[session]\nfoo=1\nboss_encounter_path=D:\\logs\\\n";
        assert_eq!(ini_log_path(ini), Some(PathBuf::from("D:\\logs\\")));
        assert_eq!(ini_log_path("[session]\nboss_encounter_path=\n"), None);
    }
}
