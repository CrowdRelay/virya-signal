use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
const sleep = (delay) => new Promise((resolve) => setTimeout(resolve, delay));
const latestInvocations = new Map();
let invocationSequence = 0;

export async function viryaInvoke(command, args, timeoutMs) {
  const timeout = Math.max(1_000, Math.min(Number(timeoutMs) || 30_000, 60_000));
  const startedAt = Date.now();
  const deadline = Date.now() + timeout;
  let core;

  // Android can expose the page a moment before the injected Tauri bridge.
  // Wait briefly instead of treating that harmless race as a broken session.
  while (!(core = window.__TAURI__?.core) && Date.now() < deadline) {
    await sleep(25);
  }
  if (!core?.invoke) throw new Error('Natywny most aplikacji nie jest dostępny.');

  const remaining = Math.max(1, deadline - Date.now());
  let timer;
  try {
    const result = await Promise.race([
      core.invoke(command, args),
      new Promise((_, reject) => {
        timer = setTimeout(
          () => reject(new Error(`Operacja ${command} przekroczyła limit czasu.`)),
          remaining,
        );
      }),
    ]);
    const elapsed = Date.now() - startedAt;
    if (elapsed >= 1_000) window.console?.info?.('[virya:ipc]', command, `${elapsed}ms`);
    return result;
  } catch (error) {
    window.console?.warn?.('[virya:ipc]', command, 'failed', `${Date.now() - startedAt}ms`, error);
    throw error;
  } finally {
    clearTimeout(timer);
  }
}

export async function viryaInvokeLatest(command, args, timeoutMs, scope) {
  const token = ++invocationSequence;
  latestInvocations.set(scope, token);
  try {
    const value = await viryaInvoke(command, args, timeoutMs);
    return latestInvocations.get(scope) === token ? value : undefined;
  } catch (error) {
    if (latestInvocations.get(scope) !== token) return undefined;
    throw error;
  }
}

export function viryaInvalidateLatest(prefix) {
  for (const scope of latestInvocations.keys()) {
    if (scope.startsWith(prefix)) latestInvocations.set(scope, ++invocationSequence);
  }
}

function viryaPermissionState(value) {
  if (typeof value === 'string') return value;
  return value?.camera ?? value?.status ?? value?.state ?? 'prompt';
}

async function viryaEnsureCameraPermission(scanner) {
  if (!scanner?.checkPermissions || !scanner?.requestPermissions) {
    throw new Error('Moduł uprawnień aparatu nie jest dostępny w tej wersji aplikacji.');
  }

  let state = viryaPermissionState(await scanner.checkPermissions());
  if (state === 'prompt' || state === 'prompt-with-rationale') {
    state = viryaPermissionState(await scanner.requestPermissions());
  }
  if (state !== 'granted') {
    throw new Error(
      'Brak dostępu do aparatu. Włącz Aparat dla Virya Signal w ustawieniach aplikacji.',
    );
  }
}

const VIRYA_SCAN_CANCELLED = '__VIRYA_SCAN_CANCELLED__';

function viryaRemoveScannerOverlay() {
  window.document?.getElementById('virya-scanner-overlay')?.remove();
  window.document?.documentElement?.removeAttribute('data-virya-scanner-active');
}

function viryaMountScannerOverlay(scanner) {
  const document = window.document;
  if (!document?.body) {
    return { cancelled: () => false, cancelPromise: new Promise(() => {}), cleanup: () => {} };
  }

  viryaRemoveScannerOverlay();
  document.documentElement.setAttribute('data-virya-scanner-active', 'true');
  const overlay = document.createElement('div');
  overlay.id = 'virya-scanner-overlay';
  overlay.setAttribute('role', 'dialog');
  overlay.setAttribute('aria-modal', 'true');
  overlay.setAttribute('aria-label', 'Skaner kodu QR');
  overlay.innerHTML = `
    <div class="virya-scanner-copy">
      <strong>SKANUJ KOD QR</strong>
      <span>Umieść kod wewnątrz ramki</span>
    </div>
    <div class="virya-scanner-frame" aria-hidden="true"></div>
    <button id="virya-scanner-cancel" type="button">← ANULUJ SKANOWANIE</button>
  `;

  let wasCancelled = false;
  let resolveCancel;
  const cancelPromise = new Promise((resolve) => { resolveCancel = resolve; });
  const cancel = overlay.querySelector('#virya-scanner-cancel');
  cancel?.addEventListener('click', () => {
    if (wasCancelled) return;
    wasCancelled = true;
    cancel.disabled = true;
    cancel.textContent = 'ZAMYKAM…';

    resolveCancel?.(VIRYA_SCAN_CANCELLED);

    const nativeCancel = () => {
      try {
        return Promise.resolve(scanner.cancel?.()).catch((error) => {
          window.console?.warn?.('[virya:scanner] cancel failed', error);
        });
      } catch (error) {
        window.console?.warn?.('[virya:scanner] cancel threw', error);
        return Promise.resolve();
      }
    };
    void nativeCancel();
    window.setTimeout(() => void nativeCancel(), 250);
  });

  document.body.appendChild(overlay);
  return {
    cancelled: () => wasCancelled,
    cancelPromise,
    cleanup: viryaRemoveScannerOverlay,
  };
}

export async function viryaScanQr() {
  const scanner = window.__TAURI__?.barcodeScanner;
  if (!scanner?.scan || !scanner?.cancel) {
    throw new Error('Skaner jest dostępny tylko w aplikacji iOS/Android.');
  }

  await viryaEnsureCameraPermission(scanner);
  const format = scanner.Format?.QRCode ?? 'QR_CODE';
  const overlay = viryaMountScannerOverlay(scanner);
  const scanPromise = Promise.resolve(scanner.scan({ windowed: true, formats: [format] }))
    .then((result) => {
      if (overlay.cancelled()) return VIRYA_SCAN_CANCELLED;
      if (typeof result === 'string') return result;
      return result?.content ?? result?.rawValue ?? result?.text ?? '';
    })
    .catch((error) => {
      if (overlay.cancelled()) return VIRYA_SCAN_CANCELLED;
      throw error;
    });

  try {
    const result = await Promise.race([scanPromise, overlay.cancelPromise]);
    if (result === VIRYA_SCAN_CANCELLED) void scanPromise.catch(() => {});
    return result;
  } finally {
    overlay.cleanup();
  }
}

function viryaRemoveCityPicker() {
  window.document?.getElementById('virya-city-picker')?.remove();
  window.document?.documentElement?.removeAttribute('data-virya-city-picker-active');
}

function viryaNormalizeCities(value) {
  if (typeof value !== 'string') {
    throw new Error('Aplikacja otrzymała nieprawidłową odpowiedź listy miast.');
  }
  if (value.length > 512_000) {
    throw new Error('Lista miast jest zbyt duża. Spróbuj ponownie później.');
  }
  try {
    value = JSON.parse(value);
  } catch {
    throw new Error('Nie udało się odczytać listy miast. Spróbuj ponownie.');
  }
  if (!Array.isArray(value)) {
    throw new Error('CrowdRelay zwrócił nieprawidłową listę miast.');
  }
  const unique = new Map();
  for (const item of value) {
    const slug = String(item?.slug ?? '').trim();
    const name = String(item?.name ?? '').trim();
    if (!slug || !name || slug.length > 128 || Array.from(name).length > 160) continue;
    const rawCount = Number(item?.fan_count ?? item?.fanCount ?? 0);
    const fanCount = Number.isFinite(rawCount) && rawCount >= 0
      ? Math.min(Math.trunc(rawCount), Number.MAX_SAFE_INTEGER)
      : 0;
    if (!unique.has(slug)) unique.set(slug, { slug, name, fanCount });
  }
  return [...unique.values()]
    .sort((a, b) => b.fanCount - a.fanCount || a.name.localeCompare(b.name, 'pl', { sensitivity: 'base' }) || a.slug.localeCompare(b.slug))
    .slice(0, 250);
}

function viryaOpenCityPicker(cities) {
  const document = window.document;
  if (!document?.body) return Promise.reject(new Error('Widok wyboru miasta nie jest dostępny.'));

  viryaRemoveCityPicker();
  document.documentElement.setAttribute('data-virya-city-picker-active', 'true');

  return new Promise((resolve) => {
    const overlay = document.createElement('div');
    overlay.id = 'virya-city-picker';
    overlay.setAttribute('role', 'dialog');
    overlay.setAttribute('aria-modal', 'true');
    overlay.setAttribute('aria-label', 'Wybierz miasto');
    overlay.innerHTML = `
      <section class="virya-city-picker-panel">
        <header>
          <div><p>VIRYA SIGNAL</p><strong>Wybierz miasto</strong></div>
          <button id="virya-city-picker-close" type="button" aria-label="Zamknij">×</button>
        </header>
        <input id="virya-city-picker-search" type="search" inputmode="search" autocomplete="off" placeholder="Szukaj miasta…" aria-label="Szukaj miasta" />
        <div id="virya-city-picker-list" role="listbox"></div>
        <p id="virya-city-picker-empty" hidden>Brak pasujących miast.</p>
        <button id="virya-city-picker-cancel" type="button">← WRÓĆ</button>
      </section>
    `;

    const list = overlay.querySelector('#virya-city-picker-list');
    const empty = overlay.querySelector('#virya-city-picker-empty');
    const search = overlay.querySelector('#virya-city-picker-search');
    let settled = false;
    const cleanup = () => {
      window.removeEventListener('keydown', onKeyDown);
      viryaRemoveCityPicker();
    };
    const finish = (value) => {
      if (settled) return;
      settled = true;
      cleanup();
      resolve(value);
    };
    const onKeyDown = (event) => { if (event.key === 'Escape') finish(null); };
    const render = (query) => {
      const needle = String(query ?? '').trim().toLocaleLowerCase('pl');
      const visible = cities
        .filter((city) => !needle || city.name.toLocaleLowerCase('pl').includes(needle))
        .slice(0, 30);
      const fragment = document.createDocumentFragment();
      for (const city of visible) {
        const button = document.createElement('button');
        button.type = 'button';
        button.setAttribute('role', 'option');
        const name = document.createElement('strong');
        name.textContent = city.name;
        const count = document.createElement('span');
        count.textContent = city.fanCount >= 25 ? `${city.fanCount} fanów` : 'Sygnał rośnie';
        button.append(name, count);
        button.addEventListener('click', () => finish({ slug: city.slug, name: city.name }));
        fragment.appendChild(button);
      }
      list.replaceChildren(fragment);
      empty.hidden = visible.length !== 0;
    };

    overlay.querySelector('#virya-city-picker-close')?.addEventListener('click', () => finish(null));
    overlay.querySelector('#virya-city-picker-cancel')?.addEventListener('click', () => finish(null));
    overlay.addEventListener('click', (event) => { if (event.target === overlay) finish(null); });
    search?.addEventListener('input', () => render(search.value));
    window.addEventListener('keydown', onKeyDown);
    document.body.appendChild(overlay);
    render('');
    window.setTimeout(() => search?.focus(), 0);
  });
}

export async function viryaPickPublicCity(apiBaseUrl) {
  const value = await viryaInvokeLatest(
    'public_cities',
    { apiBaseUrl },
    15_000,
    'public:fan-access:cities',
  );
  if (value === undefined) return null;
  const cities = viryaNormalizeCities(value);
  if (cities.length === 0) throw new Error('CrowdRelay nie zwrócił żadnych dostępnych miast.');
  return viryaOpenCityPicker(cities);
}
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = viryaInvoke)]
    async fn invoke_js(command: &str, args: JsValue, timeout_ms: u32) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = viryaInvokeLatest)]
    async fn invoke_latest_js(
        command: &str,
        args: JsValue,
        timeout_ms: u32,
        scope: &str,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = viryaInvalidateLatest)]
    fn invalidate_latest_js(prefix: &str);

    #[wasm_bindgen(catch, js_name = viryaScanQr)]
    async fn scan_qr_js() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = viryaPickPublicCity)]
    async fn pick_public_city_js(api_base_url: &str) -> Result<JsValue, JsValue>;
}

const DEFAULT_IPC_TIMEOUT_MS: u32 = 30_000;

pub async fn invoke<T, A>(command: &str, args: &A) -> Result<T, String>
where
    T: DeserializeOwned,
    A: Serialize + ?Sized,
{
    invoke_timeout(command, args, DEFAULT_IPC_TIMEOUT_MS).await
}

pub async fn invoke_timeout<T, A>(command: &str, args: &A, timeout_ms: u32) -> Result<T, String>
where
    T: DeserializeOwned,
    A: Serialize + ?Sized,
{
    let args = serde_wasm_bindgen::to_value(args).map_err(|error| error.to_string())?;
    let value = invoke_js(command, args, timeout_ms)
        .await
        .map_err(js_error)?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

/// Runs a read request in a named UI scope. Starting a newer request in the
/// same scope makes the older result disappear, preventing stale state writes.
pub async fn invoke_latest<T, A>(
    command: &str,
    args: &A,
    timeout_ms: u32,
    scope: &str,
) -> Result<Option<T>, String>
where
    T: DeserializeOwned,
    A: Serialize + ?Sized,
{
    let args = serde_wasm_bindgen::to_value(args).map_err(|error| error.to_string())?;
    let value = invoke_latest_js(command, args, timeout_ms, scope)
        .await
        .map_err(js_error)?;
    if value.is_undefined() {
        Ok(None)
    } else {
        serde_wasm_bindgen::from_value(value)
            .map(Some)
            .map_err(|error| error.to_string())
    }
}

pub fn invalidate_latest(prefix: &str) {
    invalidate_latest_js(prefix);
}

pub async fn invoke_unit<A>(command: &str, args: &A) -> Result<(), String>
where
    A: Serialize + ?Sized,
{
    let args = serde_wasm_bindgen::to_value(args).map_err(|error| error.to_string())?;
    invoke_js(command, args, DEFAULT_IPC_TIMEOUT_MS)
        .await
        .map_err(js_error)?;
    Ok(())
}

pub async fn scan_qr() -> Result<Option<String>, String> {
    const CANCELLED: &str = "__VIRYA_SCAN_CANCELLED__";
    let value = scan_qr_js().await.map_err(js_error)?;
    let value = value
        .as_string()
        .ok_or_else(|| "Skaner nie zwrócił kodu.".to_owned())?;
    if value == CANCELLED {
        return Ok(None);
    }
    let value = value.trim();
    if value.is_empty() {
        Err("Skaner nie zwrócił kodu.".to_owned())
    } else {
        Ok(Some(value.to_owned()))
    }
}

#[derive(Deserialize)]
struct CityPickerSelection {
    slug: String,
    name: String,
}

pub async fn pick_public_city(api_base_url: &str) -> Result<Option<(String, String)>, String> {
    let value = pick_public_city_js(api_base_url).await.map_err(js_error)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    let selection: CityPickerSelection =
        serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())?;
    let slug = selection.slug.trim().to_owned();
    let name = selection.name.trim().to_owned();
    if slug.is_empty()
        || slug.len() > 128
        || name.is_empty()
        || name.chars().count() > 160
        || slug.chars().any(char::is_control)
        || name.chars().any(char::is_control)
    {
        return Err("Wybrane miasto ma nieprawidłowe dane.".to_owned());
    }
    Ok(Some((slug, name)))
}

fn js_error(value: JsValue) -> String {
    value
        .as_string()
        .or_else(|| {
            js_sys::Reflect::get(&value, &JsValue::from_str("message"))
                .ok()
                .and_then(|message| message.as_string())
        })
        .or_else(|| {
            js_sys::JSON::stringify(&value)
                .ok()
                .and_then(|v| v.as_string())
        })
        .unwrap_or_else(|| "Nieznany błąd aplikacji".to_owned())
}
