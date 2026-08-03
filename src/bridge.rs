trait OptionValueOrElseExt<T> {
    fn value_or_else<F>(self, fallback: F) -> T
    where
        F: FnOnce() -> T;
}

impl<T> OptionValueOrElseExt<T> for Option<T> {
    #[allow(clippy::manual_unwrap_or)]
    fn value_or_else<F>(self, fallback: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Some(value) => value,
            None => fallback(),
        }
    }
}

use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
const sleep = (delay) => new Promise((resolve) => setTimeout(resolve, delay));
const latestInvocations = new Map();
let invocationSequence = 0;

const VIRYA_OPERATION_STORAGE_KEY = 'virya:last-operation:v3';

function viryaSafePath() {
  return `${window.location.origin}${window.location.pathname}`.slice(0, 1_000);
}

function viryaStorageRead(key, fallback = null) {
  try {
    const raw = window.localStorage?.getItem(key);
    return raw ? JSON.parse(raw) : fallback;
  } catch {
    return fallback;
  }
}

function viryaStorageWrite(key, value) {
  try {
    window.localStorage?.setItem(key, JSON.stringify(value));
    return true;
  } catch {
    return false;
  }
}

function viryaStorageRemove(key) {
  try { window.localStorage?.removeItem(key); } catch {}
}

function viryaPersistOperation(operation) {
  viryaStorageWrite(VIRYA_OPERATION_STORAGE_KEY, {
    version: 3,
    command: String(operation.command).slice(0, 160),
    startedAt: Number(operation.startedAt) || Date.now(),
    path: viryaSafePath(),
  });
}

export async function viryaInvoke(command, args, timeoutMs) {
  const timeout = Math.max(1_000, Math.min(Number(timeoutMs) || 30_000, 60_000));
  const startedAt = Date.now();
  const deadline = Date.now() + timeout;
  const operation = { command: String(command), startedAt };
  window.__VIRYA_LAST_OPERATION__ = operation;
  viryaPersistOperation(operation);
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
    if (window.__VIRYA_LAST_OPERATION__ === operation) {
      window.__VIRYA_LAST_OPERATION__ = undefined;
      viryaStorageRemove(VIRYA_OPERATION_STORAGE_KEY);
    }
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

const VIRYA_FAILURE_STORAGE_KEY = 'virya:last-runtime-failure:v2';
const VIRYA_FAILURE_HISTORY_KEY = 'virya:runtime-failure-history:v3';
const MAX_RUNTIME_FAILURES = 8;

function viryaRuntimeMessage(error) {
  if (typeof error === 'string') return error;
  if (typeof error?.message === 'string') return error.message;
  try { return JSON.stringify(error); } catch { return 'Nieznany błąd aplikacji'; }
}

function viryaRuntimeStack(error) {
  if (typeof error?.stack === 'string') return error.stack.slice(0, 12_000);
  return '';
}

function viryaBuildRuntimeReport(kind, error) {
  const operation = window.__VIRYA_LAST_OPERATION__?.command ?? '';
  return {
    version: 2,
    kind: String(kind || 'unknown'),
    message: viryaRuntimeMessage(error).slice(0, 4_000),
    stack: viryaRuntimeStack(error),
    operation: String(operation).slice(0, 160),
    occurredAt: new Date().toISOString(),
    path: viryaSafePath(),
    userAgent: String(window.navigator?.userAgent ?? '').slice(0, 1_000),
  };
}

function viryaStoreRuntimeFailure(report) {
  viryaStorageWrite(VIRYA_FAILURE_STORAGE_KEY, report);
  const current = viryaStorageRead(VIRYA_FAILURE_HISTORY_KEY, []);
  const history = [report, ...(Array.isArray(current) ? current : [])]
    .slice(0, MAX_RUNTIME_FAILURES);
  if (!viryaStorageWrite(VIRYA_FAILURE_HISTORY_KEY, history)) {
    window.console?.warn?.('[virya:crash-store]', 'failure history was not persisted');
  }
}

function viryaClearRuntimeFailure() {
  try { window.localStorage?.removeItem(VIRYA_FAILURE_STORAGE_KEY); } catch {}
}

function viryaFailureText(report) {
  const lines = [
    `Rodzaj: ${report.kind}`,
    `Czas: ${report.occurredAt}`,
    report.operation ? `Operacja: ${report.operation}` : '',
    report.path ? `Ścieżka: ${report.path}` : '',
    `Błąd: ${report.message}`,
    report.stack ? `\nStack:\n${report.stack}` : '',
  ];
  return lines.filter(Boolean).join('\n');
}

function viryaShowRuntimeFailure(report, previous = false) {
  const document = window.document;
  if (!document?.body) {
    window.setTimeout(() => viryaShowRuntimeFailure(report, previous), 50);
    return;
  }
  document.getElementById('virya-runtime-failure')?.remove();
  const node = document.createElement('section');
  node.id = 'virya-runtime-failure';
  node.setAttribute('role', 'alertdialog');
  node.setAttribute('aria-modal', 'true');
  node.innerHTML = `
    <div class="virya-runtime-failure-card">
      <p class="eyebrow">VIRYA SIGNAL / DIAGNOSTYKA</p>
      <h2>${previous ? 'Poprzednie uruchomienie zakończyło się błędem' : 'Aplikacja zatrzymała błąd'}</h2>
      <p>Nie ukrywamy awarii. Skopiuj raport i wyślij go razem z informacją, co było kliknięte.</p>
      <pre></pre>
      <div class="virya-runtime-failure-actions">
        <button type="button" data-action="copy">KOPIUJ RAPORT</button>
        <button type="button" data-action="reload">URUCHOM PONOWNIE</button>
        <button type="button" data-action="close" class="ghost">ZAMKNIJ</button>
      </div>
      <small class="copy-status"></small>
    </div>`;
  const text = viryaFailureText(report);
  const pre = node.querySelector('pre');
  if (pre) pre.textContent = text;
  node.querySelector('[data-action="copy"]')?.addEventListener('click', async () => {
    const status = node.querySelector('.copy-status');
    try {
      await window.navigator?.clipboard?.writeText(text);
      if (status) status.textContent = 'Raport skopiowany.';
    } catch {
      if (status) status.textContent = 'Przytrzymaj tekst raportu i skopiuj ręcznie.';
    }
  });
  node.querySelector('[data-action="reload"]')?.addEventListener('click', () => {
    viryaClearRuntimeFailure();
    window.location.reload();
  });
  node.querySelector('[data-action="close"]')?.addEventListener('click', () => {
    viryaClearRuntimeFailure();
    node.remove();
  });
  document.body.appendChild(node);
}

async function viryaWaitForNativeCore() {
  const deadline = Date.now() + 15_000;
  let core;
  while (!(core = window.__TAURI__?.core) && Date.now() < deadline) await sleep(50);
  return core?.invoke ? core : undefined;
}

async function viryaRecoverNativeCrash(report) {
  try {
    const core = await viryaWaitForNativeCore();
    if (!core) return;
    const previous = await core.invoke('native_crash_report');
    if (typeof previous !== 'string' || previous.trim() === '') return;
    report('native-panic', previous);
    await core.invoke('acknowledge_native_crash');
  } catch (error) {
    window.console?.warn?.('[virya:native-crash-recovery]', error);
  }
}

function viryaRecoverInterruptedOperation(report) {
  const operation = viryaStorageRead(VIRYA_OPERATION_STORAGE_KEY);
  if (!operation || typeof operation.command !== 'string') return;
  report(
    'interrupted-native-operation',
    `Poprzednie uruchomienie przerwało operację ${operation.command}.`,
  );
  viryaStorageRemove(VIRYA_OPERATION_STORAGE_KEY);
}

function viryaRecoverBootDiagnostic(report) {
  const diagnostic = window.__VIRYA_BOOT_DIAGNOSTIC__;
  if (!diagnostic) return;
  report(
    String(diagnostic.kind || 'unexpected-foreground-termination'),
    String(diagnostic.message || 'Poprzednie uruchomienie zakończyło się bez czystego zamknięcia.'),
  );
  window.__VIRYA_BOOT_DIAGNOSTIC__ = undefined;
}

export function viryaInstallRuntimeGuards() {
  if (window.__VIRYA_RUNTIME_GUARDS__) return;
  window.__VIRYA_RUNTIME_GUARDS__ = true;
  const report = (kind, error) => {
    const failure = viryaBuildRuntimeReport(kind, error);
    window.console?.error?.(`[virya:${kind}]`, failure);
    viryaStoreRuntimeFailure(failure);
    viryaShowRuntimeFailure(failure, false);
    window.dispatchEvent(new CustomEvent('virya-runtime-error', { detail: failure }));
  };
  window.addEventListener('error', (event) => report('window-error', event.error ?? event.message));
  window.addEventListener('unhandledrejection', (event) => {
    event.preventDefault();
    report('unhandled-rejection', event.reason);
  });
  viryaRecoverInterruptedOperation(report);
  viryaRecoverBootDiagnostic(report);
  void viryaRecoverNativeCrash(report);
  try {
    const raw = window.localStorage?.getItem(VIRYA_FAILURE_STORAGE_KEY);
    if (raw) {
      const previous = JSON.parse(raw);
      const age = Date.now() - Date.parse(previous?.occurredAt ?? '');
      if (Number.isFinite(age) && age >= 0 && age < 24 * 60 * 60 * 1_000) {
        viryaShowRuntimeFailure(previous, true);
      } else {
        viryaClearRuntimeFailure();
      }
    }
  } catch (error) {
    window.console?.warn?.('[virya:previous-crash]', error);
    viryaClearRuntimeFailure();
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

    #[wasm_bindgen(js_name = viryaInstallRuntimeGuards)]
    fn install_runtime_guards_js();
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

pub fn install_runtime_guards() {
    install_runtime_guards_js();
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
        .value_or_else(|| "Nieznany błąd aplikacji".to_owned())
}
