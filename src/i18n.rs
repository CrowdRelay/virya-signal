use std::sync::atomic::{AtomicU8, Ordering};

use wasm_bindgen::prelude::*;

mod en;
mod pl;

const LANGUAGE_STORAGE_KEY: &str = "virya:language:v1";
static LANGUAGE: AtomicU8 = AtomicU8::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Pl,
    En,
}

impl Language {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Pl => "pl",
            Self::En => "en",
        }
    }

    pub fn from_code(code: &str) -> Self {
        if matches!(code.as_bytes(), [b'e', b'n']) {
            Self::En
        } else {
            Self::Pl
        }
    }
}

#[wasm_bindgen(inline_js = r#"
export function viryaStoredLanguage(key) {
  try { return window.localStorage?.getItem(key) || 'pl'; } catch { return 'pl'; }
}
export function viryaSetLanguageAndReload(key, value) {
  try { window.localStorage?.setItem(key, value); } catch {}
  window.location.reload();
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = viryaStoredLanguage)]
    fn stored_language_js(key: &str) -> String;
    #[wasm_bindgen(js_name = viryaSetLanguageAndReload)]
    fn set_language_and_reload_js(key: &str, value: &str);
}

pub fn initialize() {
    set_current(Language::from_code(&stored_language_js(
        LANGUAGE_STORAGE_KEY,
    )));
}

pub fn current() -> Language {
    if LANGUAGE.load(Ordering::Relaxed) == 1 {
        Language::En
    } else {
        Language::Pl
    }
}

fn set_current(language: Language) {
    LANGUAGE.store(
        if language == Language::En { 1 } else { 0 },
        Ordering::Relaxed,
    );
}

pub fn select(language: Language) {
    if language != current() {
        set_current(language);
        set_language_and_reload_js(LANGUAGE_STORAGE_KEY, language.code());
    }
}

pub fn tr(key: &'static str) -> &'static str {
    match current() {
        Language::Pl => pl::text(key),
        Language::En => en::text(key),
    }
}

pub fn format(key: &'static str, values: &[String]) -> String {
    let mut output = tr(key).to_owned();
    for value in values {
        if let Some(start) = output.find('{')
            && let Some(relative_end) = output[start..].find('}')
        {
            output.replace_range(start..=start + relative_end, value);
        }
    }
    output
}
