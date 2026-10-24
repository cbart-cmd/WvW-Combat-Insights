//! Estado compartido del addon y hilo que vigila la carpeta de logs y analiza cada combate nuevo.

use crate::{
    evtc::{self, Fight},
    i18n::{self, Message, Text},
    logs,
    settings::Settings,
    zip,
};
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{
        Arc, LazyLock, Mutex, MutexGuard,
        mpsc::{self, RecvTimeoutError, Sender},
    },
    thread::JoinHandle,
    time::{Duration, Instant, SystemTime},
};

/// Resultado del ultimo log analizado.
#[derive(Clone)]
pub struct Report {
    pub path: PathBuf,
    pub file_name: String,
    pub fight: Fight,
    pub parse_time: Duration,
    pub live: Option<LiveStatus>,
}

#[derive(Clone)]
pub struct LiveStatus {
    pub limited: bool,
}

pub struct State {
    pub settings: Settings,
    pub history: Vec<Arc<Report>>,
    pub live: Option<Arc<Report>>,
    /// Indice del informe mostrado; el mas reciente ocupa la posicion cero.
    pub history_index: usize,
    pub status: Message,
    /// Carpeta de logs en uso.
    pub log_dir: Option<PathBuf>,
}

static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| {
    Mutex::new(State {
        settings: Settings::load(),
        history: Vec::new(),
        live: None,
        history_index: 0,
        status: Message::new(Text::Waiting, vec![]),
        log_dir: None,
    })
});

pub fn state() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

enum Msg {
    /// WvW Fight Analysis termino de procesar este archivo (solo el nombre).
    Parsed(String),
    /// Analizar el log mas reciente, aunque sea anterior a la carga del addon.
    Latest,
    /// Reprocesar los logs mas recientes segun el limite del historial.
    ReprocessHistory,
    /// Volver a analizar el log actual (han cambiado los ajustes del analisis).
    Reanalyze,
    Stop,
}

static SENDER: Mutex<Option<Sender<Msg>>> = Mutex::new(None);
static WORKER: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

const POLL: Duration = Duration::from_secs(3);

fn send(msg: Msg) {
    if let Some(tx) = SENDER.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        let _ = tx.send(msg);
    }
}

pub fn log_parsed(file_name: String) {
    send(Msg::Parsed(file_name));
}

pub fn analyze_latest() {
    send(Msg::Latest);
}

pub fn reprocess_history() {
    send(Msg::ReprocessHistory);
}

pub fn reanalyze() {
    send(Msg::Reanalyze);
}

/// Arranca el hilo de analisis. `arcdps_ini`: ruta de `arcdps.ini` para encontrar la carpeta de logs.
pub fn start(arcdps_ini: Option<PathBuf>) {
    let (tx, rx) = mpsc::channel();
    *SENDER.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
    let handle = std::thread::spawn(move || {
        let mut last_poll = Instant::now();
        let started = SystemTime::now();
        let initial_dir = logs::log_dir(&state().settings.log_dir, arcdps_ini.clone());
        if let Some((path, _)) =
            initial_dir.and_then(|dir| logs::newest(&dir, SystemTime::UNIX_EPOCH))
        {
            match read_fight(&path, true) {
                Ok(fight) => {
                    if let Some(language) = fight.language {
                        i18n::set_log_language(language);
                    }
                }
                Err(error) => nexus::log::log(
                    nexus::log::LogLevel::Warning,
                    crate::nexusbase::project::NAME,
                    format!("Could not detect language from {}: {error}", path.display()),
                ),
            }
        }
        let mut seen: HashSet<PathBuf> = HashSet::new();
        let mut pending: Option<(PathBuf, u64)> = None;
        loop {
            let msg = rx.recv_timeout(crate::live::REFRESH);
            let vs_players_only = state().settings.vs_players_only;
            crate::live::refresh(vs_players_only);
            let dir = {
                let mut st = state();
                st.log_dir = logs::log_dir(&st.settings.log_dir, arcdps_ini.clone());
                st.log_dir.clone()
            };
            match msg {
                Ok(Msg::Stop) | Err(RecvTimeoutError::Disconnected) => break,
                Ok(Msg::Parsed(name)) => {
                    if let Some(path) = dir.and_then(|d| logs::find(&d, &name))
                        && seen.insert(path.clone())
                    {
                        let _ = analyze(path, true, true);
                    }
                }
                Ok(Msg::Latest) => match dir.and_then(|d| logs::newest(&d, SystemTime::UNIX_EPOCH))
                {
                    Some((path, _)) => {
                        seen.insert(path.clone());
                        let _ = analyze(path, false, true);
                    }
                    None => state().status = Message::new(Text::NoLogs, vec![]),
                },
                Ok(Msg::ReprocessHistory) => {
                    let Some(dir) = dir else {
                        state().status = Message::new(Text::NoDir, vec![]);
                        continue;
                    };
                    let limit = state().settings.history_size;
                    let paths = logs::recent(&dir, limit);
                    if paths.is_empty() {
                        state().status = Message::new(Text::NoLogs, vec![]);
                        continue;
                    }
                    state().status = Message::new(Text::Loading, vec![]);
                    let mut loaded = 0;
                    let mut errors = Vec::new();
                    for path in paths.into_iter().rev() {
                        match analyze(path, false, true) {
                            Ok(()) => loaded += 1,
                            Err(err) => errors.push(err),
                        }
                    }
                    let mut st = state();
                    st.status = if let Some(error) = errors.first() {
                        Message::new(
                            Text::LoadErrors,
                            vec![loaded.to_string(), errors.len().to_string(), error.clone()],
                        )
                    } else {
                        Message::new(Text::Loaded, vec![loaded.to_string()])
                    };
                }
                Ok(Msg::Reanalyze) => {
                    crate::live::invalidate();
                    let paths: Vec<_> = {
                        let st = state();
                        st.history.iter().map(|r| r.path.clone()).collect()
                    };
                    for path in paths {
                        let _ = analyze(path, false, false);
                    }
                }
                Err(RecvTimeoutError::Timeout) => {
                    if last_poll.elapsed() < POLL {
                        continue;
                    }
                    last_poll = Instant::now();
                    // respaldo por si WvW Fight Analysis no esta: el log se analiza cuando deja de crecer
                    let Some((path, size)) = dir.and_then(|d| logs::newest(&d, started)) else {
                        continue;
                    };
                    if seen.contains(&path) {
                        continue;
                    }
                    if pending.as_ref() == Some(&(path.clone(), size)) && size > 0 {
                        pending = None;
                        seen.insert(path.clone());
                        let _ = analyze(path, true, true);
                    } else {
                        pending = Some((path, size));
                    }
                }
            }
        }
    });
    *WORKER.lock().unwrap_or_else(|e| e.into_inner()) = Some(handle);
}

pub fn stop() {
    send(Msg::Stop);
    SENDER.lock().unwrap_or_else(|e| e.into_inner()).take();
    if let Some(handle) = WORKER.lock().unwrap_or_else(|e| e.into_inner()).take() {
        let _ = handle.join();
    }
    crate::live::reset();
}

fn read_fight(path: &std::path::Path, vs_players_only: bool) -> Result<Fight, String> {
    std::fs::read(path)
        .map_err(|e| format!("read: {e}"))
        .and_then(|raw| zip::first_entry(&raw))
        .and_then(|bytes| evtc::parse(&bytes, vs_players_only))
}

/// Analiza un log; `auto` = llegado tras un combate (puede abrir la ventana).
fn analyze(path: PathBuf, auto: bool, select: bool) -> Result<(), String> {
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let vs_players_only = {
        let mut st = state();
        if !auto {
            st.status = Message::new(Text::Analyzing, vec![file_name.clone()]);
        }
        st.settings.vs_players_only
    };
    let start = Instant::now();
    let result = read_fight(&path, vs_players_only);
    if let Ok(fight) = &result {
        crate::live::log_finished(fight);
    }
    if let Ok(fight) = &result
        && let Some(language) = fight.language
        && let Some(dir) = state().log_dir.clone()
        && logs::newest(&dir, SystemTime::UNIX_EPOCH).is_some_and(|(latest, _)| latest == path)
    {
        i18n::set_log_language(language);
    }
    let mut st = state();
    match result {
        Ok(fight) => {
            if auto && fight.duration_secs() >= st.settings.min_duration_secs as f64 {
                let before = st.settings.clone();
                for window in &mut st.settings.windows {
                    if window.enabled && window.auto_show {
                        window.visible = true;
                    }
                }
                if st.settings != before {
                    st.settings.save();
                }
            }
            st.status = Message::default();
            let report = Arc::new(Report {
                path,
                file_name,
                fight,
                parse_time: start.elapsed(),
                live: None,
            });
            if let Some(index) = st.history.iter().position(|old| old.path == report.path) {
                if select && index != 0 {
                    st.history.remove(index);
                    st.history.insert(0, report);
                    st.history_index = 0;
                } else {
                    st.history[index] = report;
                    if select {
                        st.history_index = 0;
                    }
                }
            } else {
                st.history.insert(0, report);
                if select {
                    st.history_index = 0;
                } else {
                    st.history_index += 1;
                }
            }
            let history_size = st.settings.history_size;
            st.history.truncate(history_size);
            st.history_index = st.history_index.min(st.history.len().saturating_sub(1));
            Ok(())
        }
        Err(err) => {
            st.status = Message::new(Text::AnalysisError, vec![file_name.clone(), err.clone()]);
            Err(format!("{file_name}: {err}"))
        }
    }
}
