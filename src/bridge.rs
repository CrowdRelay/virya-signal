use serde::{de::DeserializeOwned, Serialize};
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
    return { cancelled: () => false, cleanup: () => {} };
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
  const cancel = overlay.querySelector('#virya-scanner-cancel');
  cancel?.addEventListener('click', async () => {
    if (wasCancelled) return;
    wasCancelled = true;
    cancel.disabled = true;
    cancel.textContent = 'ZAMYKAM…';
    try {
      await scanner.cancel?.();
    } catch (error) {
      window.console?.warn?.('[virya:scanner] cancel failed', error);
    }
  });

  document.body.appendChild(overlay);
  return {
    cancelled: () => wasCancelled,
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
  try {
    const result = await scanner.scan({ windowed: true, formats: [format] });
    if (overlay.cancelled()) return VIRYA_SCAN_CANCELLED;
    if (typeof result === 'string') return result;
    return result?.content ?? result?.rawValue ?? result?.text ?? '';
  } catch (error) {
    if (overlay.cancelled()) return VIRYA_SCAN_CANCELLED;
    throw error;
  } finally {
    overlay.cleanup();
  }
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
