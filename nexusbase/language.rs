//! Game language and Nexus/Unofficial Extras fallbacks, independent of addon text.

use super::project::LANGUAGE_KEY;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Language {
    Spanish,
    English,
    French,
    German,
}

impl Language {
    pub const ALL: [Self; 4] = [Self::Spanish, Self::English, Self::French, Self::German];

    pub fn from_game(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::English),
            2 => Some(Self::French),
            3 => Some(Self::German),
            4 => Some(Self::Spanish),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        ["es", "en", "fr", "de"][self as usize]
    }
}

static EXTRAS_LANGUAGE: AtomicU8 = AtomicU8::new(u8::MAX);
static LOG_LANGUAGE: AtomicU8 = AtomicU8::new(u8::MAX);

pub fn initialize() {
    EXTRAS_LANGUAGE.store(u8::MAX, Ordering::Relaxed);
    LOG_LANGUAGE.store(u8::MAX, Ordering::Relaxed);
    for language in Language::ALL {
        nexus::localization::set_translation(LANGUAGE_KEY, language.code(), language.code());
    }
    unsafe {
        nexus::event_subscribe!("EV_UNOFFICIAL_EXTRAS_LANGUAGE_CHANGED" => u32, |value| {
            if let Some(value) = value {
                set_extras_language(*value);
            }
        })
    }
    .revert_on_unload();
}

pub fn set_extras_language(value: u32) {
    if let Some(language) = Language::from_game(value) {
        EXTRAS_LANGUAGE.store(language as u8, Ordering::Relaxed);
    }
}

pub fn set_log_language(language: Language) {
    LOG_LANGUAGE.store(language as u8, Ordering::Relaxed);
}

pub fn game_language() -> Option<Language> {
    nexus::rtapi::RealTimeApi::get()
        .and_then(|api| api.read_game())
        .and_then(|game| game.language.ok())
        .and_then(|language| Language::from_game(language as u32))
}

pub fn has_game_language() -> bool {
    game_language().is_some()
        || Language::ALL
            .get(EXTRAS_LANGUAGE.load(Ordering::Relaxed) as usize)
            .is_some()
}

pub fn select_language(
    game: Option<Language>,
    extras: u8,
    log: u8,
    nexus: impl FnOnce() -> Option<String>,
) -> Language {
    if let Some(language) = game {
        return language;
    }
    for value in [extras, log] {
        if let Some(language) = Language::ALL.get(value as usize) {
            return *language;
        }
    }
    let code = nexus();
    Language::ALL
        .into_iter()
        .find(|lang| Some(lang.code()) == code.as_deref())
        .unwrap_or(Language::English)
}

pub fn current() -> Language {
    select_language(
        game_language(),
        EXTRAS_LANGUAGE.load(Ordering::Relaxed),
        LOG_LANGUAGE.load(Ordering::Relaxed),
        || nexus::localization::translate(LANGUAGE_KEY),
    )
}

pub fn format_template(template: &str, args: &[String]) -> String {
    let mut result = String::new();
    let mut rest = template;
    while let Some(at) = rest.find('{') {
        result.push_str(&rest[..at]);
        rest = &rest[at..];
        if let Some(end) = rest.find('}')
            && let Ok(index) = rest[1..end].parse::<usize>()
            && let Some(value) = args.get(index)
        {
            result.push_str(value);
            rest = &rest[end + 1..];
        } else {
            result.push('{');
            rest = &rest[1..];
        }
    }
    result.push_str(rest);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_priority_and_lazy_fallback_are_stable() {
        for (id, expected) in [
            (0, Language::English),
            (2, Language::French),
            (3, Language::German),
            (4, Language::Spanish),
        ] {
            assert_eq!(Language::from_game(id), Some(expected));
            assert_eq!(
                select_language(Some(expected), 0, 0, || panic!("unneeded fallback")),
                expected
            );
        }
        assert_eq!(Language::from_game(1), None);
        assert_eq!(
            select_language(None, 2, 0, || panic!("unneeded fallback")),
            Language::French
        );
        assert_eq!(
            select_language(None, u8::MAX, 3, || panic!("unneeded fallback")),
            Language::German
        );
        assert_eq!(
            select_language(None, u8::MAX, u8::MAX, || Some("es".into())),
            Language::Spanish
        );
        assert_eq!(
            select_language(None, u8::MAX, u8::MAX, || None),
            Language::English
        );
    }

    #[test]
    fn formatting_preserves_unknown_placeholders_and_literal_arguments() {
        assert_eq!(
            format_template("{1}: {0}/{1}", &["{1}".into(), "value".into()]),
            "value: {1}/value"
        );
        assert_eq!(format_template("{9} {bad} {", &[]), "{9} {bad} {");
    }
}
