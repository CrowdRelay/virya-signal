use std::sync::atomic::{AtomicU8, Ordering};

// Both the WASM UI and the native Tauri core compile the same static catalogs.
// This keeps translations in one PL/EN pair without runtime JSON or hash maps.
#[path = "../../src/i18n/en.rs"]
mod en;
#[path = "../../src/i18n/pl.rs"]
mod pl;

static LANGUAGE: AtomicU8 = AtomicU8::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Language {
    Pl,
    En,
}

impl Language {
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::Pl => "pl",
            Self::En => "en",
        }
    }

    fn from_code(code: &str) -> Self {
        if code.eq_ignore_ascii_case("en") {
            Self::En
        } else {
            Self::Pl
        }
    }
}

pub(crate) fn set_language(code: &str) {
    let value = if Language::from_code(code) == Language::En {
        1
    } else {
        0
    };
    LANGUAGE.store(value, Ordering::Relaxed);
}

pub(crate) fn current() -> Language {
    if LANGUAGE.load(Ordering::Relaxed) == 1 {
        Language::En
    } else {
        Language::Pl
    }
}

pub(crate) fn tr(key: &'static str) -> &'static str {
    match current() {
        Language::Pl => pl::text(key),
        Language::En => en::text(key),
    }
}

pub(crate) fn replace(template_key: &'static str, values: &[(&str, String)]) -> String {
    let mut output = tr(template_key).to_owned();
    for (name, value) in values {
        output = output.replace(&format!("{{{name}}}"), value);
    }
    output
}
