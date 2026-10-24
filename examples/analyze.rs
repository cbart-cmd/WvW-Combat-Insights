//! Analiza un `.zevtc` desde la linea de comandos, sin el juego:
//! `cargo run --example analyze -- ["ruta\al\log.zevtc"] [top] [es|en|fr|de]`
//! Sin ruta usa el log mas reciente de la carpeta de arcdps.

use fight_analysis_danger::{danger, evtc, i18n::Language, logs, zip};

fn main() {
    let mut args = std::env::args().skip(1).peekable();
    let path = match args.peek() {
        Some(a) if a.parse::<usize>().is_err() => args.next().unwrap().into(),
        _ => {
            let ini =
                std::path::PathBuf::from(r"C:\Program Files\Guild Wars 2\addons\arcdps\arcdps.ini");
            let dir = logs::log_dir("", Some(ini)).expect("sin carpeta de logs");
            println!("carpeta de logs: {}", dir.display());
            logs::newest(&dir, std::time::SystemTime::UNIX_EPOCH)
                .expect("no hay logs")
                .0
        }
    };
    println!("log: {}", path.display());
    let top = args.next().and_then(|n| n.parse().ok()).unwrap_or(2);
    let language_override = args.next().map(|code| {
        Language::ALL
            .into_iter()
            .find(|language| language.code() == code)
            .expect("idioma no soportado: usa es, en, fr o de")
    });
    let raw = std::fs::read(&path).expect("no se pudo leer el log");
    let start = std::time::Instant::now();
    let bytes = zip::first_entry(&raw).expect("no se pudo descomprimir");
    let fight = evtc::parse(&bytes, true).expect("no se pudo analizar");
    let language = language_override
        .or(fight.language)
        .unwrap_or(Language::English);
    println!("idioma: {}", language.code());
    println!(
        "{} jugadores, {:.0}s de combate, color propio {:?}, analizado en {:?}",
        fight.players.len(),
        fight.duration_secs(),
        fight.pov_team,
        start.elapsed()
    );
    for team in evtc::Team::ALL {
        let members = fight
            .players
            .iter()
            .filter(|p| p.team == Some(team))
            .count();
        let events = evtc::team_event_counts(&fight, team);
        println!(
            "  {}: {} jugadores, {} kills, {} muertes, {} derribos",
            team.name(),
            members,
            events.kills,
            events.deaths,
            events.downs
        );
    }
    println!(
        "healing: {:?}, {} miembros de escuadron identificados",
        fight.healing_coverage,
        fight
            .players
            .iter()
            .filter(|p| fight.is_squad_member(p))
            .count()
    );
    let ranks = danger::rank(&fight, top);
    print!("{}", danger::summary_in(&fight, &ranks, language));
    if std::env::var_os("VERBOSE").is_some() {
        for p in &fight.players {
            println!(
                "{:?} {:?} {:?} {} {} {} healing={} ({:?}) barrier={} ({:?}) squad={} weapons={:?} underwater={:?}",
                p.team,
                p.name,
                p.account,
                language.spec(p.profession, p.elite),
                p.damage,
                p.strips,
                p.healing,
                p.healing_coverage,
                p.barrier,
                p.barrier_coverage,
                p.squad_member,
                p.weapons,
                p.underwater_weapons
            );
        }
    }
}
