use std::{
    cell::RefCell,
    sync::atomic::{AtomicU8, Ordering},
};

use wasm_bindgen::prelude::*;

const LANGUAGE_STORAGE_KEY: &str = "virya:language:v1";
static LANGUAGE: AtomicU8 = AtomicU8::new(0);

thread_local! {
    // Web copy lives in boot-i18n.js rather than the WASM data section. Keep the
    // tiny page-lifetime cache as a sorted Vec instead of pulling hash-table
    // machinery into the release WASM. Lookups stay allocation-free after the
    // first JS crossing and Box::leak remains bounded by both language catalogs.
    static TRANSLATION_CACHE: RefCell<Vec<((&'static str, &'static str), &'static str)>> =
        const { RefCell::new(Vec::new()) };
}

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
export function viryaSetLanguage(key, value) {
  try { window.localStorage?.setItem(key, value); } catch {}
  const runtime = globalThis.__VIRYA_RUNTIME_I18N__;
  if (runtime?.requestLanguage) runtime.requestLanguage(value);
  else try { window.dispatchEvent(new CustomEvent('virya:language-change')); } catch {}
}
export function viryaRuntimeText(language, key) {
  const value = globalThis.__VIRYA_RUNTIME_I18N__?.text?.(language, key);
  return typeof value === 'string' ? value : key;
}
export function viryaRuntimeI18nReady() {
  return globalThis.__VIRYA_RUNTIME_I18N__?.ready?.() ?? Promise.resolve();
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = viryaStoredLanguage)]
    fn stored_language_js(key: &str) -> String;
    #[wasm_bindgen(js_name = viryaSetLanguage)]
    fn set_language_js(key: &str, value: &str);
    #[wasm_bindgen(js_name = viryaRuntimeText)]
    fn runtime_text_js(language: &str, key: &str) -> String;
    #[wasm_bindgen(catch, js_name = viryaRuntimeI18nReady)]
    async fn runtime_i18n_ready_js() -> Result<JsValue, JsValue>;
}

pub fn initialize() {
    set_current(Language::from_code(&stored_language_js(
        LANGUAGE_STORAGE_KEY,
    )));
}

/// The selected catalog is a JSON asset, kept outside the WASM module. Mount
/// only after it has arrived so English users never see a Polish/key fallback
/// during the first render; a failed non-default fetch falls back in the
/// loader before this promise resolves.
pub async fn wait_for_runtime_catalog() {
    let _ = runtime_i18n_ready_js().await;
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
        set_language_js(LANGUAGE_STORAGE_KEY, language.code());
    }
}

pub fn tr(key: &'static str) -> &'static str {
    let language = current().code();
    let cache_key = (language, key);
    let cached = TRANSLATION_CACHE.with(|cache| {
        let cache = cache.borrow();
        cache
            .binary_search_by_key(&cache_key, |(stored_key, _)| *stored_key)
            .ok()
            .map(|index| cache[index].1)
    });
    if let Some(value) = cached {
        return value;
    }

    let value = runtime_text_js(language, key);
    // The runtime catalog is fetched, so `tr` can be called before it has
    // arrived. The JS side answers a miss with the key itself, and caching that
    // froze the raw identifier for the life of the WebView: whichever labels
    // happened to render during the load showed as `type` or `checklist_tab`
    // and never recovered, even though the catalog held a translation. A miss
    // is not an answer — return it for this render and ask again next time.
    if value == key {
        return key;
    }
    let value: &'static str = Box::leak(value.into_boxed_str());
    TRANSLATION_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        match cache.binary_search_by_key(&cache_key, |(stored_key, _)| *stored_key) {
            Ok(index) => cache[index].1 = value,
            Err(index) => cache.insert(index, (cache_key, value)),
        }
    });
    value
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
