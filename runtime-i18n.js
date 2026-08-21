(() => {
  "use strict";

  const VERSION = "0.4.2-runtime-i18n-v3";
  const LANGUAGE_STORAGE_KEY = "virya:language:v1";
  const catalogs = Object.create(null);
  const pending = Object.create(null);
  let keys = null;

  const language = (value) => value === "en" ? "en" : "pl";
  const asset = (name) => `${name}?v=${VERSION}`;
  const readJson = async (name) => {
    const response = await fetch(asset(name), { cache: "force-cache", credentials: "same-origin" });
    if (!response.ok) throw new Error(`i18n asset failed: ${response.status}`);
    return response.json();
  };
  const indexOf = (key) => {
    let low = 0;
    let high = keys.length - 1;
    while (low <= high) {
      const middle = (low + high) >>> 1;
      const candidate = keys[middle];
      if (candidate === key) return middle;
      if (candidate < key) low = middle + 1;
      else high = middle - 1;
    }
    return -1;
  };
  const load = (requested) => {
    const selected = language(requested);
    if (catalogs[selected]) return Promise.resolve(selected);
    if (pending[selected]) return pending[selected];
    const task = (async () => {
      const [loadedKeys, values] = await Promise.all([
      keys ? Promise.resolve(keys) : readJson("runtime-i18n-keys.json"),
      readJson(`runtime-i18n-${selected}.json`),
      ]);
      if (!Array.isArray(loadedKeys) || !Array.isArray(values) || loadedKeys.length !== values.length
        || !loadedKeys.every((key) => typeof key === "string")
        || !values.every((value) => typeof value === "string")) {
        throw new Error("invalid i18n catalog");
      }
      keys = loadedKeys;
      catalogs[selected] = values;
      return selected;
    })();
    pending[selected] = task;
    void task.finally(() => { delete pending[selected]; });
    return task;
  };
  const preferred = () => {
    try { return language(window.localStorage?.getItem(LANGUAGE_STORAGE_KEY)); } catch { return "pl"; }
  };
  const dispatchReady = () => {
    try { window.dispatchEvent(new CustomEvent("virya:language-change")); } catch {}
  };
  const loadWithFallback = (requested) => load(requested).catch(() => load("pl"));
  let ready = loadWithFallback(preferred());

  window.__VIRYA_RUNTIME_I18N__ = Object.freeze({
    ready: () => ready,
    requestLanguage(requested) {
      const request = loadWithFallback(requested);
      ready = request;
      void request.then(() => { if (ready === request) dispatchReady(); });
      return request;
    },
    text(requested, key) {
      if (!keys) return key;
      const values = catalogs[language(requested)] || catalogs.pl;
      const index = values ? indexOf(key) : -1;
      return index >= 0 ? values[index] : key;
    },
  });
})();
