//! Automatic game language and addon translations. Game names come from the official API.

use crate::nexusbase::language::format_template;
pub use crate::nexusbase::language::{Language, initialize, set_extras_language, set_log_language};
use crate::{evtc::Team, nexusbase::gw2::game_names};

impl Language {
    pub fn text(self, key: Text) -> &'static str {
        key.translations()[self as usize]
    }

    pub fn format(self, key: Text, args: &[String]) -> String {
        format_template(self.text(key), args)
    }

    pub fn widget(self, key: Text) -> String {
        format!("{}###FAD_{key:?}", self.text(key))
    }

    pub fn team(self, team: Team) -> &'static str {
        self.text(match team {
            Team::Red => Text::Red,
            Team::Blue => Text::Blue,
            Team::Green => Text::Green,
            Team::Own => Text::OurTeam,
        })
    }

    pub fn spec(self, profession: u32, elite: u32) -> &'static str {
        game_names::SPECIALIZATIONS
            .iter()
            .find(|(id, _)| *id == elite)
            .or_else(|| {
                game_names::PROFESSIONS
                    .iter()
                    .find(|(id, _)| *id == profession)
            })
            .map(|(_, names)| names[self as usize])
            .unwrap_or_else(|| self.text(Text::Unknown))
    }

    pub fn weapon(self, id: &str) -> &str {
        // The API's weapon_type enum is language-independent, not a localized name.
        WEAPONS
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, names)| names[self as usize])
            .unwrap_or(id)
    }

    pub fn rank_name<'a>(self, name: &'a str) -> &'a str {
        game_names::WVW_RANKS
            .iter()
            .find(|(_, names)| names.contains(&name))
            .map(|(_, names)| names[self as usize])
            .unwrap_or(name)
    }
}

pub fn current() -> Language {
    use windows::Win32::UI::{
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_RCONTROL},
        WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
    };

    let right_control = unsafe { GetAsyncKeyState(i32::from(VK_RCONTROL.0)) } < 0;
    let focused = right_control
        && unsafe {
            let mut process = 0;
            GetWindowThreadProcessId(GetForegroundWindow(), Some(&mut process));
            process == std::process::id()
        };
    temporary_language(right_control, focused, crate::nexusbase::language::current)
}

fn temporary_language(
    right_control: bool,
    focused: bool,
    normal: impl FnOnce() -> Language,
) -> Language {
    if right_control && focused {
        Language::English
    } else {
        normal()
    }
}

#[cfg(test)]
fn select_language(
    game: Option<Language>,
    extras: u8,
    log: u8,
    nexus: impl FnOnce() -> Option<String>,
) -> Language {
    crate::nexusbase::language::select_language(game, extras, log, nexus)
}

pub fn format_duration(duration_ms: u64, language: Language) -> String {
    let total_seconds = duration_ms / 1_000 + u64::from(duration_ms % 1_000 >= 500);
    let hours = total_seconds / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;
    let mut parts = Vec::with_capacity(3);
    if hours > 0 {
        parts.push(language.format(Text::Hours, &[hours.to_string()]));
    }
    if minutes > 0 {
        parts.push(language.format(Text::Minutes, &[minutes.to_string()]));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(language.format(Text::Seconds, &[seconds.to_string()]));
    }
    parts.join(" ")
}

pub fn format_log_name(file_name: &str, language: Language) -> String {
    let Some(timestamp) = parse_log_timestamp(file_name) else {
        return file_name.to_owned();
    };
    let now = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
    format_log_timestamp(timestamp, (now.wYear, now.wMonth, now.wDay), language)
}

#[derive(Clone, Copy)]
struct LogTimestamp {
    year: u16,
    month: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
}

fn parse_log_timestamp(file_name: &str) -> Option<LogTimestamp> {
    let name = file_name.rsplit(['\\', '/']).next()?;
    let timestamp = name.get(..15)?;
    if name.len() > 15 && !name[15..].starts_with('.') {
        return None;
    }
    if !timestamp
        .bytes()
        .enumerate()
        .all(|(index, byte)| match index {
            8 => byte == b'-',
            _ => byte.is_ascii_digit(),
        })
    {
        return None;
    }
    let parse = |range: std::ops::Range<usize>| timestamp[range].parse().ok();
    let parsed = LogTimestamp {
        year: parse(0..4)?,
        month: parse(4..6)?,
        day: parse(6..8)?,
        hour: parse(9..11)?,
        minute: parse(11..13)?,
        second: parse(13..15)?,
    };
    (valid_date(parsed.year, parsed.month, parsed.day)
        && parsed.hour < 24
        && parsed.minute < 60
        && parsed.second < 60)
        .then_some(parsed)
}

fn valid_date(year: u16, month: u16, day: u16) -> bool {
    if year == 0 || !(1..=12).contains(&month) {
        return false;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_month = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=days_in_month).contains(&day)
}

fn day_number(year: u16, month: u16, day: u16) -> u32 {
    let year = u32::from(year);
    let leap_days = |year| year / 4 - year / 100 + year / 400;
    let days_before_year = |year| 365 * (year - 1) + leap_days(year - 1);
    let month_days = match month {
        1 => 0,
        2 => 31,
        3 => 59,
        4 => 90,
        5 => 120,
        6 => 151,
        7 => 181,
        8 => 212,
        9 => 243,
        10 => 273,
        11 => 304,
        12 => 334,
        _ => unreachable!(),
    };
    days_before_year(year)
        + month_days
        + u32::from(
            month > 2
                && year.is_multiple_of(4)
                && (!year.is_multiple_of(100) || year.is_multiple_of(400)),
        )
        + u32::from(day - 1)
}

fn format_log_timestamp(
    timestamp: LogTimestamp,
    today: (u16, u16, u16),
    language: Language,
) -> String {
    let time = format!(
        "{:02}:{:02}:{:02}",
        timestamp.hour, timestamp.minute, timestamp.second
    );
    match day_number(timestamp.year, timestamp.month, timestamp.day)
        .abs_diff(day_number(today.0, today.1, today.2))
    {
        0 if day_number(timestamp.year, timestamp.month, timestamp.day)
            == day_number(today.0, today.1, today.2) =>
        {
            match language {
                Language::Spanish => format!("Hoy a las {time}"),
                Language::English => format!("Today at {time}"),
                Language::French => format!("Aujourd'hui à {time}"),
                Language::German => format!("Heute um {time}"),
            }
        }
        1 if day_number(timestamp.year, timestamp.month, timestamp.day)
            < day_number(today.0, today.1, today.2) =>
        {
            match language {
                Language::Spanish => format!("Ayer a las {time}"),
                Language::English => format!("Yesterday at {time}"),
                Language::French => format!("Hier à {time}"),
                Language::German => format!("Gestern um {time}"),
            }
        }
        _ => {
            let date = match language {
                Language::Spanish | Language::French => format!(
                    "{:02}/{:02}/{:04}",
                    timestamp.day, timestamp.month, timestamp.year
                ),
                Language::English => format!(
                    "{:04}-{:02}-{:02}",
                    timestamp.year, timestamp.month, timestamp.day
                ),
                Language::German => format!(
                    "{:02}.{:02}.{:04}",
                    timestamp.day, timestamp.month, timestamp.year
                ),
            };
            let separator = match language {
                Language::Spanish => "a las",
                Language::English => "at",
                Language::French => "à",
                Language::German => "um",
            };
            format!("{date} {separator} {time}")
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Message {
    key: Option<Text>,
    args: Vec<String>,
}

impl Message {
    pub fn new(key: Text, args: Vec<String>) -> Self {
        Self {
            key: Some(key),
            args,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.key.is_none()
    }

    pub fn is_history_loaded(&self) -> bool {
        matches!(self.key, Some(Text::Loaded))
    }

    pub fn render(&self, language: Language) -> String {
        self.key
            .map(|key| {
                let mut args = self.args.clone();
                if matches!(key, Text::AnalysisError | Text::LoadErrors)
                    && let Some(error) = args.last_mut()
                {
                    *error = diagnostic(language, error);
                }
                language.format(key, &args)
            })
            .unwrap_or_default()
    }
}

macro_rules! translations {
    ($($key:ident => [$es:expr, $en:expr, $fr:expr, $de:expr]),* $(,)?) => {
        #[derive(Clone, Copy, Debug)]
        pub enum Text { $($key),* }
        impl Text {
            pub const ALL: &'static [Self] = &[$(Self::$key),*];
            fn translations(self) -> [&'static str; 4] {
                match self { $(Self::$key => [$es, $en, $fr, $de]),* }
            }
        }
    };
}

translations! {
    OurTeam => ["Nuestro equipo", "Our team", "Notre équipe", "Unser Team"],
    LiveNoTeams => ["Colores no disponibles en los eventos recibidos.", "Team colors unavailable in received events.", "Couleurs indisponibles dans les événements reçus.", "Teamfarben in empfangenen Ereignissen nicht verfügbar."],
    LiveLimit => ["Directo incompleto: límite de eventos", "Live incomplete: event limit", "Direct incomplet : limite d'événements", "Live unvollständig: Ereignislimit"],
    Live => ["En directo · provisional", "Live · provisional", "En direct · provisoire", "Live · vorläufig"],
    LiveTip => ["Solo nuestro equipo y eventos recibidos de arcdps; cobertura parcial. El daño usa equipos identificados o afinidad hostil, sin exigir colores. Curas/barrera requieren Healing Stats compatible. Fuera de combate se muestra el último log.", "Our team and received arcdps events only; partial coverage. Damage uses identified teams or hostile affinity without requiring colors. Healing/barrier require compatible Healing Stats. Outside combat the latest log is shown.", "Notre équipe et événements arcdps reçus uniquement ; couverture partielle. Dégâts selon équipes identifiées ou affinité hostile, sans exiger les couleurs. Soins/barrières : Healing Stats compatible requis. Hors combat : dernier journal.", "Nur unser Team und empfangene arcdps-Ereignisse; teilweise Abdeckung. Schaden nutzt Team-IDs oder feindliche Zuordnung ohne Farbpflicht. Heilung/Barriere benötigen kompatible Healing Stats. Außerhalb des Kampfes: letztes Log."],
    HistoryMenu => ["Historial", "History", "Historique", "Verlauf"],
    StatsMenu => ["Estadísticas", "Stats", "Statistiques", "Statistiken"],
    StyleMenu => ["Estilo", "Style", "Style", "Stil"],
    PositionMenu => ["Posición", "Position", "Position", "Position"],
    InverseSort => ["Invertir orden", "Inverse sort order", "Inverser le tri", "Sortierung umkehren"],
    Opacity => ["Opacidad (%)", "Opacity (%)", "Opacité (%)", "Deckkraft (%)"],
    ResetStyle => ["Tamaño y opacidad por defecto", "Default size and opacity", "Taille et opacité par défaut", "Standardgröße und Deckkraft"],
    PlayersShort => ["Jug.", "Players", "Joueurs", "Spieler"],
    HighlightsShort => ["Destacados", "Highlights", "Joueurs", "Spieler"],
    PartialShort => ["Parcial", "Partial", "Partiel", "Teilweise"],
    ContextTip => ["Clic derecho: historial y opciones", "Right-click: history and options", "Clic droit : historique et options", "Rechtsklick: Verlauf und Optionen"],
    UnderwaterWeapon => ["Arma usada bajo el agua", "Weapon used underwater", "Arme utilisée sous l'eau", "Unter Wasser verwendete Waffe"],
    Overview => ["Resumen", "Overview", "Résumé", "Übersicht"],
    Highlights => ["Jugadores destacados", "Player highlights", "Joueurs remarquables", "Hervorgehobene Spieler"],
    Healing => ["Healing", "Healing", "Soins", "Heilung"],
    HealingAmount => ["Curas", "Healing", "Soins", "Heilung"],
    Barrier => ["Barrera", "Barrier", "Barrière", "Barriere"],
    Cleanses => ["Limpias", "Cleanses", "Purifications", "Reinigungen"],
    HighlightScore => ["Daño + corrups × 2000 + Down contribution", "Damage + strips × 2000 + Down contribution", "Dégâts + dissipations × 2000 + Contribution aux mises à terre", "Schaden + Entfernungen × 2000 + Niederschlagsbeitrag"],
    DownContribution => ["Down contribution", "Down contribution", "Contribution aux mises à terre", "Niederschlagsbeitrag"],
    DownContributionShort => ["down c", "down c", "down c", "down c"],
    CleanseTip => ["Limpias registradas al escuadrón (incluye propias): una por condición retirada por completo, no por acumulación. No requiere Healing Stats.", "Recorded squad cleanses (including self): one per fully removed condition, not per stack. Does not require Healing Stats.", "Purifications enregistrées de l'escouade (soi-même inclus) : une par altération entièrement retirée, pas par charge. Sans Healing Stats.", "Aufgezeichnete Einsatztrupp-Reinigungen (einschließlich selbst): eine pro vollständig entfernter Bedingung, nicht pro Stapel. Ohne Healing Stats."],
    Total => ["Total", "Total", "Total", "Gesamt"],
    Kills => ["Kills", "Kills", "Éliminations", "Tötungen"],
    Player => ["Jugador", "Player", "Joueur", "Spieler"],
    NoActivity => ["Sin jugadores con daño o strips registrados en este equipo.", "No players with recorded damage or strips on this team.", "Aucun joueur avec des dégâts ou dissipations enregistrés dans cette équipe.", "Keine Spieler mit aufgezeichnetem Schaden oder Entfernungen in diesem Team."],
    Enabled => ["Habilitar ventana", "Enable window", "Activer la fenêtre", "Fenster aktivieren"],
    FixedPosition => ["Posición fija X / Y", "Fixed X / Y position", "Position X / Y fixe", "Feste X-/Y-Position"],
    LockWindow => ["Bloquear movimiento", "Lock movement", "Verrouiller le déplacement", "Verschieben sperren"],
    Width => ["Ancho (px)", "Width (px)", "Largeur (px)", "Breite (px)"],
    Height => ["Alto (px)", "Height (px)", "Hauteur (px)", "Hohe (px)"],
    Columns => ["Columnas", "Columns", "Colonnes", "Spalten"],
    ResetPosition => ["Restablecer posición", "Reset position", "Réinitialiser la position", "Position zurücksetzen"],
    FollowLatest => ["Seguir el último combate", "Follow latest fight", "Suivre le dernier combat", "Neuestem Kampf folgen"],
    InitialTeam => ["Equipo inicial", "Initial team", "Équipe initiale", "Anfangsteam"],
    FirstEnemy => ["Primer enemigo", "First enemy", "Premier ennemi", "Erster Gegner"],
    OwnTeamShort => ["Tu equipo", "Your team", "Votre équipe", "Dein Team"],
    ShowWeapons => ["Mostrar armas", "Show weapons", "Afficher les armes", "Waffen anzeigen"],
    Rows => ["Filas", "Rows", "Lignes", "Zeilen"],
    SortBy => ["Orden inicial", "Initial order", "Tri initial", "Anfangssortierung"],
    ShowBars => ["Mostrar barras comparativas", "Show comparison bars", "Afficher les barres comparatives", "Vergleichsbalken anzeigen"],
    GeneralOptions => ["Análisis e historial (compartidos)", "Analysis and history (shared)", "Analyse et historique (partagés)", "Analyse und Verlauf (gemeinsam)"],
    OpenAnother => ["Abrir otra ventana", "Open another window", "Ouvrir une autre fenêtre", "Weiteres Fenster öffnen"],
    PinnedFight => ["Combate fijado", "Pinned fight", "Combat épinglé", "Angehefteter Kampf"],
    HealingMissing => ["Este log no contiene datos compatibles de healing.", "This log has no supported healing data.", "Ce journal ne contient pas de données de soins compatibles.", "Dieses Protokoll enthält keine unterstützten Heilungsdaten."],
    HealingPartial => ["Datos parciales: solo se muestran aportaciones registradas.", "Partial data: only recorded contributions are shown.", "Données partielles : seules les contributions enregistrées sont affichées.", "Unvollständige Daten: Nur aufgezeichnete Beiträge werden angezeigt."],
    HealingNoGroup => ["No se puede identificar el grupo o escuadrón en este log.", "The party or squad cannot be identified in this log.", "Le groupe ou l'escouade ne peut pas être identifié dans ce journal.", "Gruppe oder Einsatztrupp sind in diesem Protokoll nicht erkennbar."],
    HealingEmpty => ["Sin curas ni barreras registradas para el grupo o escuadrón.", "No healing or barrier recorded for the party or squad.", "Aucun soin ni barrière enregistré pour le groupe ou l'escouade.", "Keine Heilung oder Barriere für Gruppe oder Einsatztrupp aufgezeichnet."],
    HealingScope => ["Grupo / escuadrón: curas y barreras registradas, no curación efectiva. No compara jugadores sin datos.", "Party / squad: recorded healing and barrier, not effective healing. Players without data are not compared.", "Groupe / escouade : soins et barrières enregistrés, pas les soins effectifs. Les joueurs sans données ne sont pas comparés.", "Gruppe / Einsatztrupp: aufgezeichnete Heilung und Barriere, keine effektive Heilung. Spieler ohne Daten werden nicht verglichen."],
    HealingRecorded => ["Curas y barreras registradas (no efectivas)", "Recorded healing and barrier (not effective)", "Soins et barrières enregistrés (non effectifs)", "Aufgezeichnete Heilung und Barriere (nicht effektiv)"],
    Subgroup => ["Subgrupo", "Subgroup", "Sous-groupe", "Untergruppe"],
    PerSecond => ["Por segundo", "Per second", "Par seconde", "Pro Sekunde"],
    TeamEventsTip => ["{0}\nJugadores registrados: {1}\nKills contra jugadores: {2}\nMuertes: {3}\nDerribos: {4}", "{0}\nRecorded players: {1}\nKills against players: {2}\nDeaths: {3}\nDowns: {4}", "{0}\nJoueurs enregistrés : {1}\nÉliminations de joueurs : {2}\nMorts : {3}\nMises à terre : {4}", "{0}\nErfasste Spieler: {1}\nTötungen von Spielern: {2}\nTode: {3}\nNiederschläge: {4}"],
    Title => ["Enemigos peligrosos", "Dangerous enemies", "Ennemis dangereux", "Gefährliche Gegner"],
    Reprocess => ["Reprocesar últimos {0} combates", "Reprocess last {0} fights", "Réanalyser les {0} derniers combats", "Letzte {0} Kämpfe neu analysieren"],
    Latest => ["Analizar último log", "Analyze latest log", "Analyser le dernier journal", "Neuestes Protokoll analysieren"],
    Previous => ["< Anterior", "< Previous", "< Précédent", "< Zurück"],
    Next => ["Siguiente >", "Next >", "Suivant >", "Weiter >"],
    FightIndex => ["Combate {0} de {1}", "Fight {0} of {1}", "Combat {0} sur {1}", "Kampf {0} von {1}"],
    EnemySummary => ["Enemigos: {0}", "Enemies: {0}", "Ennemis : {0}", "Gegner: {0}"],
    TeamSummary => ["Resumen por color", "Team summary", "Résumé par couleur", "Übersicht nach Team"],
    Team => ["Color", "Team", "Couleur", "Team"],
    Players => ["Jugadores", "Players", "Joueurs", "Spieler"],
    Deaths => ["Muertes", "Deaths", "Morts", "Tode"],
    Downs => ["Derribos", "Downs", "Mises à terre", "Niederschläge"],
    Copy => ["Copiar", "Copy", "Copier", "Kopieren"],
    CopyTip => ["Copia el ranking completo al portapapeles", "Copy the full ranking to the clipboard", "Copier le classement complet dans le presse-papiers", "Die gesamte Rangliste in die Zwischenablage kopieren"],
    Chat => ["Chat", "Chat", "Discussion", "Chat"],
    ChatTip => ["Copia el resumen de todos los colores; un color por línea", "Copy the summary of all colors; one color per line", "Copier le résumé de toutes les couleurs ; une couleur par ligne", "Die Zusammenfassung aller Farben kopieren; eine Farbe pro Zeile"],
    FightInfo => ["{0} - {1}, {2} jugadores{3}", "{0} - {1}, {2} players{3}", "{0} - {1}, {2} joueurs{3}", "{0} - {1}, {2} Spieler{3}"],
    OwnTeam => [", tu color: {0}", ", your color: {0}", ", votre couleur : {0}", ", deine Farbe: {0}"],
    Hours => ["{0} h", "{0} h", "{0} h", "{0} Std."],
    Minutes => ["{0} min", "{0} min", "{0} min", "{0} Min."],
    Seconds => ["{0} s", "{0} s", "{0} s", "{0} s"],
    RedTotal => ["Rojo {0}", "Red {0}", "Rouge {0}", "Rot {0}"],
    GreenTotal => ["Verde {0}", "Green {0}", "Vert {0}", "Grün {0}"],
    BlueTotal => ["Azul {0}", "Blue {0}", "Bleu {0}", "Blau {0}"],
    OtherAllies => ["Aliados {0}", "Allies {0}", "Alliés {0}", "Verbündete {0}"],
    RosterTip => [
        "Totales por color de los jugadores registrados en el log, incluidos los de fuera de tu grupo o squad y tú en tu color. Aliados: jugadores de tu color en otros subgrupos conocidos, excluye tu subgrupo; no es el total de tu squad. ? indica datos no disponibles. El subgrupo 0 puede ser ambiguo.",
        "Color totals count players recorded in the log, including those outside your party or squad and yourself in your color. Allies: players of your color in other known subgroups, excluding your subgroup; not your full squad. ? means unavailable data. Subgroup 0 may be ambiguous.",
        "Les totaux par couleur comptent les joueurs du journal, y compris hors de votre groupe ou escouade et vous-même dans votre couleur. Alliés : joueurs de votre couleur dans les autres sous-groupes connus, sans votre sous-groupe ; pas toute votre escouade. ? indique des données indisponibles. Le sous-groupe 0 peut être ambigu.",
        "Farbgesamtzahlen zählen die im Protokoll erfassten Spieler, auch außerhalb deiner Gruppe oder deines Einsatztrupps und dich in deiner Farbe. Verbündete: Spieler deiner Farbe in anderen bekannten Untergruppen, ohne deine Untergruppe; nicht dein gesamter Einsatztrupp. ? bedeutet fehlende Daten. Untergruppe 0 kann mehrdeutig sein."
    ],
    ParseTime => ["Analizado en {0} s\n{1}", "Analyzed in {0} s\n{1}", "Analysé en {0} s\n{1}", "In {0} s analysiert\n{1}"],
    NoEnemies => ["No hay enemigos con actividad en este log.", "No enemies with recorded activity in this log.", "Aucun ennemi avec une activité enregistrée dans ce journal.", "Keine Gegner mit aufgezeichneter Aktivität in diesem Protokoll."],
    Active => ["{0} ({1} activos)", "{0} ({1} active)", "{0} ({1} actifs)", "{0} ({1} aktiv)"],
    Specialization => ["Especialización", "Specialization", "Spécialisation", "Spezialisierung"],
    Weapons => ["Armas (estimadas)", "Weapons (estimated)", "Armes (estimées)", "Waffen (geschätzt)"],
    Rank => ["Rango WvW", "WvW rank", "Rang McM", "WvW-Rang"],
    Dps => ["DPS", "DPS", "DPS", "DPS"],
    Damage => ["Daño", "Damage", "Dégâts", "Schaden"],
    Strips => ["Corrups", "Strips", "Dissipations", "Entfernungen"],
    StripPosition => ["Strip {0}", "Strip {0}", "Dissipation {0}", "Entfernung {0}"],
    NotDetected => ["No detectadas", "Not detected", "Non détectées", "Nicht erkannt"],
    WeaponTip => ["Inferidas de las habilidades de arma usadas; pueden faltar armas que no se usaron.", "Inferred from weapon skills used; unused weapons may be missing.", "Déduites des compétences d'arme utilisées ; les armes non utilisées peuvent manquer.", "Aus verwendeten Waffenfertigkeiten abgeleitet; unbenutzte Waffen können fehlen."],
    DamageTip => ["Directo: {0}\nCondiciones: {1}", "Strike: {0}\nConditions: {1}", "Directs : {0}\nAltérations : {1}", "Direkt: {0}\nZustände: {1}"],
    Description => ["Resumen, jugadores destacados y healing de tus combates WvW", "Overview, player highlights and healing for your WvW fights", "Résumé, joueurs remarquables et soins de vos combats McM", "Übersicht, hervorgehobene Spieler und Heilung deiner WvW-Kämpfe"],
    WvwOnly => ["La ventana solo se muestra en mapas de WvW (siempre).", "The window is only shown on WvW maps (always).", "La fenêtre s'affiche uniquement sur les cartes McM (toujours).", "Das Fenster wird nur auf WvW-Karten angezeigt (immer)."],
    Visible => ["Visible ahora", "Visible now", "Visible maintenant", "Jetzt sichtbar"],
    AutoShow => ["Abrir la ventana sola al terminar un combate", "Open window automatically after a fight", "Ouvrir automatiquement la fenêtre après un combat", "Fenster nach einem Kampf automatisch öffnen"],
    HideInCombat => ["Ocultar la ventana durante el combate", "Hide window during combat", "Masquer la fenêtre pendant le combat", "Fenster im Kampf ausblenden"],
    PlayersOnly => ["Contar solo daño y strips sobre jugadores", "Only count damage and strips against players", "Compter uniquement les dégâts et dissipations contre les joueurs", "Nur Schaden und Entfernungen gegen Spieler zählen"],
    TopDps => ["Jugadores por DPS", "Players by DPS", "Joueurs par DPS", "Spieler nach DPS"],
    TopStrips => ["Jugadores por strips", "Players by strips", "Joueurs par dissipations", "Spieler nach Entfernungen"],
    HistorySize => ["Combates en historial", "Fights in history", "Combats dans l'historique", "Kämpfe im Verlauf"],
    MinimumDuration => ["Duración mínima para abrir (s)", "Minimum duration to open (s)", "Durée minimale pour ouvrir (s)", "Mindestdauer zum Öffnen (s)"],
    LogDir => ["Carpeta de logs (vacío = la de arcdps)", "Log folder (empty = arcdps default)", "Dossier des journaux (vide = celui d'arcdps)", "Protokollordner (leer = arcdps-Standard)"],
    CurrentDir => ["Carpeta en uso: {0}", "Current folder: {0}", "Dossier utilisé : {0}", "Aktueller Ordner: {0}"],
    Undetermined => ["(sin determinar)", "(undetermined)", "(indéterminé)", "(unbekannt)"],
    Shortcut => ["Atajo: ALT+SHIFT+P (se cambia en Nexus)", "Shortcut: ALT+SHIFT+P (change in Nexus)", "Raccourci : ALT+MAJ+P (modifiable dans Nexus)", "Tastenkürzel: ALT+UMSCHALT+P (in Nexus ändern)"],
    HiddenNames => ["WvW oculta nombre, cuenta y guild de los enemigos; solo se ve su rango.", "WvW hides enemy names, accounts and guilds; only their rank is visible.", "Le McM masque les noms, comptes et guildes des ennemis ; seul leur rang est visible.", "WvW verbirgt Namen, Accounts und Gilden der Gegner; nur ihr Rang ist sichtbar."],
    Red => ["Rojo", "Red", "Rouge", "Rot"],
    Blue => ["Azul", "Blue", "Bleu", "Blau"],
    Green => ["Verde", "Green", "Vert", "Grün"],
    Unknown => ["Desconocido", "Unknown", "Inconnu", "Unbekannt"],
    Waiting => ["Esperando al próximo combate...", "Waiting for the next fight...", "En attente du prochain combat...", "Warte auf den nächsten Kampf..."],
    NoLogs => ["No se encontraron logs en la carpeta de arcdps.", "No logs found in the arcdps folder.", "Aucun journal trouvé dans le dossier d'arcdps.", "Keine Protokolle im arcdps-Ordner gefunden."],
    NoDir => ["No se ha podido determinar la carpeta de logs.", "Could not determine the log folder.", "Impossible de déterminer le dossier des journaux.", "Der Protokollordner konnte nicht ermittelt werden."],
    Loading => ["Cargando los últimos logs...", "Loading recent logs...", "Chargement des derniers journaux...", "Lade die neuesten Protokolle..."],
    Loaded => ["Historial: {0} combates cargados.", "History: {0} fights loaded.", "Historique : {0} combats chargés.", "Verlauf: {0} Kämpfe geladen."],
    LoadErrors => ["Historial: {0} cargados; {1} errores. {2}", "History: {0} loaded; {1} errors. {2}", "Historique : {0} chargés ; {1} erreurs. {2}", "Verlauf: {0} geladen; {1} Fehler. {2}"],
    Analyzing => ["Analizando {0}...", "Analyzing {0}...", "Analyse de {0}...", "Analysiere {0}..."],
    AnalysisError => ["{0}: no se pudo analizar el log. {1}", "{0}: could not analyze the log. {1}", "{0} : impossible d'analyser le journal. {1}", "{0}: Protokoll konnte nicht analysiert werden. {1}"],
    ReadError => ["No se pudo leer el archivo: {0}", "Could not read the file: {0}", "Impossible de lire le fichier : {0}", "Datei konnte nicht gelesen werden: {0}"],
    SummaryRow => ["{0} {1}: {2} - {3} dps, {4} strips\n", "{0} {1}: {2} - {3} dps, {4} strips\n", "{0} {1} : {2} - {3} dps, {4} dissipations\n", "{0} {1}: {2} - {3} dps, {4} Entfernungen\n"],
    ChatPlayer => ["{0} {1} dps, {2} strips", "{0} {1} dps, {2} strips", "{0} {1} dps, {2} dissipations", "{0} {1} dps, {2} Entfernungen"],
}

const WEAPONS: &[(&str, [&str; 4])] = &[
    ("Axe", ["Hacha", "Axe", "Hache", "Axt"]),
    ("Dagger", ["Daga", "Dagger", "Dague", "Dolch"]),
    ("Focus", ["Foco", "Focus", "Focus", "Fokus"]),
    (
        "Greatsword",
        ["Mandoble", "Greatsword", "Espadon", "Großschwert"],
    ),
    ("Hammer", ["Martillo", "Hammer", "Marteau", "Hammer"]),
    (
        "Longbow",
        ["Arco largo", "Longbow", "Arc long", "Langbogen"],
    ),
    ("Mace", ["Maza", "Mace", "Masse", "Streitkolben"]),
    ("Pistol", ["Pistola", "Pistol", "Pistolet", "Pistole"]),
    ("Rifle", ["Rifle", "Rifle", "Fusil", "Gewehr"]),
    ("Scepter", ["Cetro", "Scepter", "Sceptre", "Zepter"]),
    ("Shield", ["Escudo", "Shield", "Bouclier", "Schild"]),
    (
        "Shortbow",
        ["Arco corto", "Shortbow", "Arc court", "Kurzbogen"],
    ),
    ("Spear", ["Lanza", "Spear", "Lance", "Speer"]),
    (
        "Speargun",
        ["Arpón", "Speargun", "Fusil-harpon", "Harpunenschleuder"],
    ),
    ("Staff", ["Báculo", "Staff", "Bâton", "Stab"]),
    ("Sword", ["Espada", "Sword", "Épée", "Schwert"]),
    ("Torch", ["Antorcha", "Torch", "Torche", "Fackel"]),
    ("Trident", ["Tridente", "Trident", "Trident", "Dreizack"]),
    (
        "Warhorn",
        ["Cuerno de guerra", "Warhorn", "Cor de guerre", "Kriegshorn"],
    ),
];

fn diagnostic(language: Language, error: &str) -> String {
    if let Some((_, texts)) = DIAGNOSTICS.iter().find(|(source, _)| *source == error) {
        return texts[language as usize].into();
    }
    for (prefix, suffix, texts) in [
        (
            "revision EVTC ",
            " no soportada",
            [
                "Revisión EVTC {0} no soportada",
                "Unsupported EVTC revision {0}",
                "Révision EVTC {0} non prise en charge",
                "EVTC-Version {0} nicht unterstützt",
            ],
        ),
        (
            "zip: metodo de compresion ",
            " no soportado",
            [
                "zip: método de compresión {0} no soportado",
                "zip: unsupported compression method {0}",
                "zip : méthode de compression {0} non prise en charge",
                "zip: Kompressionsmethode {0} nicht unterstützt",
            ],
        ),
    ] {
        if let Some(value) = error
            .strip_prefix(prefix)
            .and_then(|value| value.strip_suffix(suffix))
        {
            return format_template(texts[language as usize], &[value.into()]);
        }
    }
    if let Some(detail) = error.strip_prefix("read: ") {
        return language.format(Text::ReadError, &[detail.into()]);
    }
    if let Some((file, detail)) = error.split_once(": ") {
        return format!("{file}: {}", diagnostic(language, detail));
    }
    error.into()
}

const DIAGNOSTICS: &[(&str, [&str; 4])] = &[
    (
        "no es un log EVTC",
        [
            "No es un log EVTC",
            "Not an EVTC log",
            "Ce n'est pas un journal EVTC",
            "Kein EVTC-Protokoll",
        ],
    ),
    (
        "no es un log de WvW",
        [
            "No es un log de WvW",
            "Not a WvW log",
            "Ce n'est pas un journal McM",
            "Kein WvW-Protokoll",
        ],
    ),
    (
        "log truncado",
        [
            "Log truncado",
            "Truncated log",
            "Journal tronqué",
            "Abgeschnittenes Protokoll",
        ],
    ),
    (
        "log truncado en agentes",
        [
            "Log truncado en agentes",
            "Truncated agent table",
            "Table des agents tronquée",
            "Abgeschnittene Agententabelle",
        ],
    ),
    (
        "log truncado en habilidades",
        [
            "Log truncado en habilidades",
            "Truncated skill table",
            "Table des compétences tronquée",
            "Abgeschnittene Fertigkeitentabelle",
        ],
    ),
    (
        "zip: directorio central no encontrado",
        [
            "zip: directorio central no encontrado",
            "zip: central directory not found",
            "zip : répertoire central introuvable",
            "zip: Zentralverzeichnis nicht gefunden",
        ],
    ),
    (
        "zip: cabecera local invalida",
        [
            "zip: cabecera local inválida",
            "zip: invalid local header",
            "zip : en-tête local invalide",
            "zip: ungültiger lokaler Header",
        ],
    ),
    (
        "zip: truncado",
        [
            "zip: truncado",
            "zip: truncated",
            "zip : tronqué",
            "zip: abgeschnitten",
        ],
    ),
    (
        "zip: datos truncados",
        [
            "zip: datos truncados",
            "zip: truncated data",
            "zip : données tronquées",
            "zip: abgeschnittene Daten",
        ],
    ),
    (
        "deflate: datos truncados",
        [
            "deflate: datos truncados",
            "deflate: truncated data",
            "deflate : données tronquées",
            "deflate: abgeschnittene Daten",
        ],
    ),
    (
        "deflate: codigo Huffman sobresuscrito",
        [
            "deflate: código Huffman sobresuscrito",
            "deflate: oversubscribed Huffman code",
            "deflate : code Huffman sursouscrit",
            "deflate: überbelegter Huffman-Code",
        ],
    ),
    (
        "deflate: codigo Huffman invalido",
        [
            "deflate: código Huffman inválido",
            "deflate: invalid Huffman code",
            "deflate : code Huffman invalide",
            "deflate: ungültiger Huffman-Code",
        ],
    ),
    (
        "deflate: longitud invalida",
        [
            "deflate: longitud inválida",
            "deflate: invalid length",
            "deflate : longueur invalide",
            "deflate: ungültige Länge",
        ],
    ),
    (
        "deflate: distancia invalida",
        [
            "deflate: distancia inválida",
            "deflate: invalid distance",
            "deflate : distance invalide",
            "deflate: ungültiger Abstand",
        ],
    ),
    (
        "deflate: distancia fuera de rango",
        [
            "deflate: distancia fuera de rango",
            "deflate: distance out of range",
            "deflate : distance hors limites",
            "deflate: Abstand außerhalb des Bereichs",
        ],
    ),
    (
        "deflate: datos descomprimidos demasiado grandes",
        [
            "deflate: datos descomprimidos demasiado grandes",
            "deflate: decompressed data too large",
            "deflate : données décompressées trop volumineuses",
            "deflate: entpackte Daten zu groß",
        ],
    ),
    (
        "deflate: cabecera dinamica invalida",
        [
            "deflate: cabecera dinámica inválida",
            "deflate: invalid dynamic header",
            "deflate : en-tête dynamique invalide",
            "deflate: ungültiger dynamischer Header",
        ],
    ),
    (
        "deflate: repeticion sin longitud previa",
        [
            "deflate: repetición sin longitud previa",
            "deflate: repetition without a previous length",
            "deflate : répétition sans longueur précédente",
            "deflate: Wiederholung ohne vorherige Länge",
        ],
    ),
    (
        "deflate: demasiadas longitudes",
        [
            "deflate: demasiadas longitudes",
            "deflate: too many lengths",
            "deflate : trop de longueurs",
            "deflate: zu viele Längen",
        ],
    ),
    (
        "deflate: falta el codigo de fin de bloque",
        [
            "deflate: falta el código de fin de bloque",
            "deflate: missing end-of-block code",
            "deflate : code de fin de bloc manquant",
            "deflate: Blockendecode fehlt",
        ],
    ),
    (
        "deflate: bloque almacenado truncado",
        [
            "deflate: bloque almacenado truncado",
            "deflate: truncated stored block",
            "deflate : bloc stocké tronqué",
            "deflate: abgeschnittener gespeicherter Block",
        ],
    ),
    (
        "deflate: longitud de bloque almacenado invalida",
        [
            "deflate: longitud de bloque almacenado inválida",
            "deflate: invalid stored block length",
            "deflate : longueur de bloc stocké invalide",
            "deflate: ungültige Länge des gespeicherten Blocks",
        ],
    ),
    (
        "deflate: tipo de bloque invalido",
        [
            "deflate: tipo de bloque inválido",
            "deflate: invalid block type",
            "deflate : type de bloc invalide",
            "deflate: ungültiger Blocktyp",
        ],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_control_temporarily_overrides_every_language_only_while_focused() {
        for normal in Language::ALL {
            assert_eq!(temporary_language(false, true, || normal), normal);
            assert_eq!(
                temporary_language(true, true, || panic!("must not query saved language")),
                Language::English
            );
            assert_eq!(temporary_language(false, true, || normal), normal);
            assert_eq!(temporary_language(true, false, || normal), normal);
            assert_eq!(temporary_language(false, false, || normal), normal);
        }
    }

    #[test]
    fn game_language_ids_match_gw2_not_os_locale() {
        assert_eq!(Language::from_game(0), Some(Language::English));
        assert_eq!(Language::from_game(2), Some(Language::French));
        assert_eq!(Language::from_game(3), Some(Language::German));
        assert_eq!(Language::from_game(4), Some(Language::Spanish));
        assert_eq!(Language::from_game(1), None);
        assert_eq!(Language::from_game(u32::MAX), None);
    }

    #[test]
    fn fight_duration_is_compact_and_splits_hours_minutes_and_seconds() {
        let spanish = Language::Spanish;
        assert_eq!(format_duration(0, spanish), "0 s");
        assert_eq!(format_duration(59_000, spanish), "59 s");
        assert_eq!(format_duration(60_000, spanish), "1 min");
        assert_eq!(format_duration(300_000, spanish), "5 min");
        assert_eq!(format_duration(301_000, spanish), "5 min 1 s");
        assert_eq!(format_duration(3_661_000, spanish), "1 h 1 min 1 s");
        assert_eq!(format_duration(3_600_000, Language::German), "1 Std.");
    }

    #[test]
    fn log_name_formats_today_yesterday_and_older_dates() {
        let today = (2026, 10, 18);
        let timestamp = parse_log_timestamp("20261018-153209.zevtc").unwrap();
        assert_eq!(
            format_log_timestamp(timestamp, today, Language::Spanish),
            "Hoy a las 15:32:09"
        );

        let yesterday = parse_log_timestamp("20261017-153209.zevtc").unwrap();
        assert_eq!(
            format_log_timestamp(yesterday, today, Language::English),
            "Yesterday at 15:32:09"
        );

        let older = parse_log_timestamp("20260929-153209.zevtc").unwrap();
        assert_eq!(
            format_log_timestamp(older, today, Language::German),
            "29.09.2026 um 15:32:09"
        );
    }

    #[test]
    fn log_name_keeps_unrecognized_or_invalid_names_unchanged() {
        for name in [
            "unrecognized.zevtc",
            "20260229-153209.zevtc",
            "20261018-253209.zevtc",
            "20261018-153209-extra.zevtc",
        ] {
            assert_eq!(format_log_name(name, Language::Spanish), name);
        }
    }

    #[test]
    fn live_game_language_takes_priority_over_extras_logs_and_nexus() {
        assert_eq!(
            select_language(
                Some(Language::German),
                Language::French as u8,
                Language::Spanish as u8,
                || panic!("Nexus fallback must not be read")
            ),
            Language::German
        );
        assert_eq!(
            select_language(
                None,
                Language::French as u8,
                Language::Spanish as u8,
                || panic!("Nexus fallback must not be read")
            ),
            Language::French
        );
        assert_eq!(
            select_language(None, u8::MAX, Language::Spanish as u8, || panic!(
                "Nexus fallback must not be read"
            )),
            Language::Spanish
        );
        assert_eq!(
            select_language(None, u8::MAX, u8::MAX, || Some("de".into())),
            Language::German
        );
        assert_eq!(
            select_language(None, u8::MAX, u8::MAX, || None),
            Language::English
        );
    }

    #[test]
    fn every_translation_has_the_same_placeholders_and_stable_widget_id() {
        for &key in Text::ALL {
            let reference = Language::English.text(key);
            for language in Language::ALL {
                let text = language.text(key);
                assert!(!text.is_empty());
                for index in 0..8 {
                    let placeholder = format!("{{{index}}}");
                    assert_eq!(
                        text.matches(&placeholder).count(),
                        reference.matches(&placeholder).count(),
                        "{key:?} {language:?}"
                    );
                }
                assert_eq!(
                    language.widget(key).split_once("###").unwrap().1,
                    Language::English.widget(key).split_once("###").unwrap().1
                );
            }
        }
        assert_eq!(
            format_template("{0} {1}", &["file{1}.evtc".into(), "2".into()]),
            "file{1}.evtc 2"
        );
    }

    #[test]
    fn statuses_and_errors_are_localized_when_rendered_not_when_created() {
        let message = Message::new(
            Text::AnalysisError,
            vec!["file.evtc".into(), "revision EVTC 2 no soportada".into()],
        );
        assert!(
            message
                .render(Language::English)
                .contains("Unsupported EVTC revision 2")
        );
        assert!(message.render(Language::French).contains("Révision EVTC 2"));
        assert!(message.render(Language::German).contains("EVTC-Version 2"));
        assert!(
            message
                .render(Language::Spanish)
                .contains("Revisión EVTC 2")
        );
        for (source, names) in DIAGNOSTICS {
            for language in Language::ALL {
                assert_eq!(diagnostic(language, source), names[language as usize]);
            }
        }
        assert_eq!(
            diagnostic(Language::English, "file.evtc: zip: datos truncados"),
            "file.evtc: zip: truncated data"
        );
        assert!(
            diagnostic(
                Language::German,
                "zip: metodo de compresion 99 no soportado"
            )
            .contains("99")
        );
        assert!(diagnostic(Language::French, "read: OS error 5").contains("OS error 5"));
    }

    #[test]
    fn game_catalog_and_weapon_labels_cover_all_languages() {
        assert_eq!(Language::French.spec(1, 27), "Draconnier");
        assert_eq!(Language::German.spec(1, 0), "Wächter");
        for language in Language::ALL {
            for (_, names) in game_names::PROFESSIONS
                .iter()
                .chain(game_names::SPECIALIZATIONS)
                .chain(game_names::WVW_RANKS)
            {
                assert!(!names[language as usize].is_empty());
            }
            for (weapon, _) in WEAPONS {
                assert!(!language.weapon(weapon).is_empty());
            }
        }
        assert_eq!(
            Language::English.rank_name("Coronel de mithril"),
            "Mithril Colonel"
        );
        assert_eq!(
            Language::French.rank_name("Actual character name"),
            "Actual character name"
        );
    }
}
