#[wasm_bindgen(inline_js = r#"
const sleep = (delay) => new Promise((resolve) => setTimeout(resolve, delay));
const latestInvocations = new Map();
let invocationSequence = 0;

let viryaTexts = {
  nativeBridgeUnavailable: 'The native app bridge is unavailable.',
  operationTimeout: 'Operation {command} timed out.',
  cameraModuleUnavailable: 'The camera permission module is unavailable in this app version.',
  cameraDenied: 'Camera access is denied. Enable Camera for Virya Signal in the app settings.',
  locationModuleUnavailable: 'The location module is unavailable in this app version.',
  locationDenied: 'Location access is denied. Enable Location for Virya Signal in the app settings.',
  locationUnavailable: 'Unfortunately, this is not the correct location. Keep looking!',
  scannerLabel: 'QR code scanner', scannerTitle: 'SCAN QR CODE', scannerHint: 'Place the code inside the frame', scannerCancel: '← CANCEL SCANNING', scannerClosing: 'CLOSING…', scannerUnavailable: 'The scanner is available only in the iOS/Android app.',
  unknownError: 'Unknown application error', reportType: 'Type', reportTime: 'Time', reportOperation: 'Operation', reportPath: 'Path', reportError: 'Error',
  diagnostics: 'VIRYA SIGNAL / DIAGNOSTICS', previousFailure: 'The previous launch ended with an error', currentFailure: 'The app caught an error', reportHelp: 'We do not hide failures. Copy the report and send it with a note about what you tapped.', copyReport: 'COPY REPORT', restart: 'RESTART APP', close: 'CLOSE', reportCopied: 'Report copied.', copyManually: 'Press and hold the report text and copy it manually.', interrupted: 'The previous launch interrupted operation {command}.', uncleanShutdown: 'The previous launch ended without a clean shutdown.'
};
export function viryaSetRuntimeTranslations(value) { if (value && typeof value === 'object') viryaTexts = { ...viryaTexts, ...value }; }
const viryaTemplate = (text, name, value) => String(text).replace(`{${name}}`, String(value));

const VIRYA_OPERATION_STORAGE_KEY = 'virya:last-operation:v3';
const VIRYA_FAN_TAB_STORAGE_KEY = 'virya:fan-tab:v1';
const VIRYA_ROOT_MODE_STORAGE_KEY = 'virya:root-mode:v1';

export function viryaReadFanTab() {
  try {
    return String(window.sessionStorage?.getItem(VIRYA_FAN_TAB_STORAGE_KEY) ?? 'signal');
  } catch {
    return 'signal';
  }
}

export function viryaWriteFanTab(value) {
  const safe = ['signal', 'events', 'merch', 'game', 'wallet', 'profile'].includes(String(value))
    ? String(value)
    : 'signal';
  try { window.sessionStorage?.setItem(VIRYA_FAN_TAB_STORAGE_KEY, safe); } catch {}
}

export function viryaReadRootMode() {
  try {
    const value = String(window.localStorage?.getItem(VIRYA_ROOT_MODE_STORAGE_KEY) ?? 'fan');
    return value === 'latarnik' ? 'latarnik' : 'fan';
  } catch {
    return 'fan';
  }
}

export function viryaWriteRootMode(value) {
  const safe = String(value) === 'latarnik' ? 'latarnik' : 'fan';
  try { window.localStorage?.setItem(VIRYA_ROOT_MODE_STORAGE_KEY, safe); } catch {}
}


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

export function viryaNativeBridgeAvailable() { return Boolean(window.__TAURI__?.core?.invoke); }

export function viryaReferralCodeFromLocation() {
  try {
    const value = new URL(window.location.href).searchParams.get('ref')?.trim() ?? '';
    return /^[A-Za-z0-9_-]{2,64}$/.test(value) ? value : '';
  } catch {
    return '';
  }
}

function viryaAsPromise(value) {
  return value && typeof value.then === 'function' ? value : Promise.resolve(value);
}

export function viryaCopyText(text) {
  const safeText = String(text ?? '').slice(0, 4_096);
  return Promise.resolve().then(async () => {
    if (window.navigator?.clipboard?.writeText) {
      await viryaAsPromise(window.navigator.clipboard.writeText(safeText));
      return 'copied';
    }
    const document = window.document;
    if (!document?.body) throw new Error('clipboard_unavailable');
    const textarea = document.createElement('textarea');
    textarea.value = safeText;
    textarea.setAttribute('readonly', '');
    textarea.style.position = 'fixed';
    textarea.style.opacity = '0';
    textarea.style.pointerEvents = 'none';
    document.body.appendChild(textarea);
    textarea.select();
    const copied = document.execCommand?.('copy') === true;
    textarea.remove();
    if (!copied) throw new Error('clipboard_unavailable');
    return 'copied';
  });
}

export function viryaShareText(title, text, url) {
  const safeTitle = String(title ?? '').slice(0, 160);
  const safeText = String(text ?? '').slice(0, 500);
  const safeUrl = String(url ?? '').slice(0, 2_048);
  // Always return a real Promise. Some Android/Tauri WebViews expose native
  // share/clipboard shims that synchronously return undefined, while the
  // wasm-bindgen async ABI expects a thenable.
  return Promise.resolve().then(async () => {
    if (window.navigator?.share) {
      try {
        await viryaAsPromise(window.navigator.share({ title: safeTitle, text: safeText, url: safeUrl }));
        return 'shared';
      } catch (error) {
        if (error?.name === 'AbortError') return 'cancelled';
        window.console?.warn?.('[virya:share] native share failed, using clipboard fallback', error);
      }
    }
    await viryaCopyText([safeText, safeUrl].filter(Boolean).join('\n'));
    return 'copied';
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
  if (!core?.invoke) throw new Error(viryaTexts.nativeBridgeUnavailable);

  const remaining = Math.max(1, deadline - Date.now());
  let timer;
  try {
    const result = await Promise.race([
      core.invoke(command, args),
      new Promise((_, reject) => {
        timer = setTimeout(
          () => reject(new Error(viryaTemplate(viryaTexts.operationTimeout, 'command', command))),
          remaining,
        );
      }),
    ]);
    const elapsed = Date.now() - startedAt;
    if (elapsed >= 1_000) window.console?.info?.('[virya:ipc]', command, `${elapsed}ms`);
    return result;
  } catch (error) {
    window.console?.warn?.('[virya:ipc]', command, 'failed', `${Date.now() - startedAt}ms`, error);
    const msg = typeof error === 'string' ? error : error?.message ?? '';
    if (msg.includes('native=panic') || msg.includes('native panic')) {
      const report = viryaBuildRuntimeReport('native-panic', error);
      report.operation = command;
      viryaStoreRuntimeFailure(report);
      viryaShowRuntimeFailure(report, false);
    }
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
  } finally {
    // Scopes are short-lived request identities, not a cache. Removing the
    // winning token prevents long sessions from retaining every visited view.
    if (latestInvocations.get(scope) === token) latestInvocations.delete(scope);
  }
}

export function viryaInvalidateLatest(prefix) {
  for (const scope of latestInvocations.keys()) {
    // Deletion is enough to make every outstanding token stale and also keeps
    // the registry bounded after logout, account switch and tab changes.
    if (scope.startsWith(prefix)) latestInvocations.delete(scope);
  }
}

function viryaPermissionState(value) {
  if (typeof value === 'string') return value;
  return value?.camera ?? value?.status ?? value?.state ?? 'prompt';
}

async function viryaEnsureCameraPermission(scanner) {
  if (!scanner?.checkPermissions || !scanner?.requestPermissions) {
    throw new Error(viryaTexts.cameraModuleUnavailable);
  }

  let state = viryaPermissionState(await scanner.checkPermissions());
  if (state === 'prompt' || state === 'prompt-with-rationale') {
    state = viryaPermissionState(await scanner.requestPermissions());
  }
  if (state !== 'granted') {
    throw new Error(
      viryaTexts.cameraDenied,
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
  overlay.setAttribute('aria-label', viryaTexts.scannerLabel);
  overlay.innerHTML = `
    <div class="virya-scanner-copy">
      <strong>${viryaTexts.scannerTitle}</strong>
      <span>${viryaTexts.scannerHint}</span>
    </div>
    <div class="virya-scanner-frame" aria-hidden="true"></div>
    <button id="virya-scanner-cancel" type="button">${viryaTexts.scannerCancel}</button>
  `;

  let wasCancelled = false;
  let resolveCancel;
  const cancelPromise = new Promise((resolve) => { resolveCancel = resolve; });
  const cancel = overlay.querySelector('#virya-scanner-cancel');
  cancel?.addEventListener('click', () => {
    if (wasCancelled) return;
    wasCancelled = true;
    cancel.disabled = true;
    cancel.textContent = viryaTexts.scannerClosing;

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

async function viryaScanQrRaw() {
  const scanner = window.__TAURI__?.barcodeScanner;
  if (!scanner?.scan || !scanner?.cancel) {
    throw new Error(viryaTexts.scannerUnavailable);
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

export async function viryaScanQr() {
  return viryaScanQrRaw();
}

export async function viryaScanAndConfirmFan() {
  const core = window.__TAURI__?.core;
  if (!core?.invoke) throw new Error(viryaTexts.nativeBridgeUnavailable);

  let result;
  try {
    result = await viryaScanQrRaw();
  } catch (error) {
    await core.invoke('fan_clear_pending_confirmation').catch(() => {});
    throw error;
  }

  if (result === VIRYA_SCAN_CANCELLED) {
    await core.invoke('fan_clear_pending_confirmation').catch(() => {});
    return null;
  }

  const token = String(result ?? '').trim();
  if (!token) {
    await core.invoke('fan_clear_pending_confirmation').catch(() => {});
    throw new Error(viryaTexts.scannerUnavailable);
  }

  const confirmed = await core.invoke('fan_confirm_scanned', { token });
  // A camera-resume launcher read can race this native commit. Reconcile once
  // more only after native state is authoritative; a disposed FanAccess owner
  // is no longer required to finish navigation.
  viryaWriteFanTab('signal');
  window.dispatchEvent(new Event('virya:resume'));
  return confirmed;
}


export async function viryaScanAndConfirmBeacon() {
  const core = window.__TAURI__?.core;
  if (!core?.invoke) throw new Error(viryaTexts.nativeBridgeUnavailable);

  let result;
  try {
    result = await viryaScanQrRaw();
  } catch (error) {
    await core.invoke('beacon_clear_pending_confirmation').catch(() => {});
    throw error;
  }

  if (result === VIRYA_SCAN_CANCELLED) {
    await core.invoke('beacon_clear_pending_confirmation').catch(() => {});
    return null;
  }

  const token = String(result ?? '').trim();
  if (!token) {
    await core.invoke('beacon_clear_pending_confirmation').catch(() => {});
    throw new Error(viryaTexts.scannerUnavailable);
  }

  const confirmed = await core.invoke('beacon_confirm_scanned', { token });
  // Like fan confirmation, native Stronghold state is committed before camera
  // resume can remount the WebView. The UI only reconciles afterwards.
  window.dispatchEvent(new Event('virya:resume'));
  return confirmed;
}


function viryaLocationStates(value) {
  if (typeof value === 'string') {
    return { precise: value, coarse: value };
  }
  const fallback = value?.status ?? value?.state ?? 'prompt';
  return {
    precise: value?.location ?? fallback,
    coarse: value?.coarseLocation ?? value?.location ?? fallback,
  };
}

async function viryaEnsureLocationPermission(core, preciseRequired = false) {
  if (!core?.invoke) throw new Error(viryaTexts.locationModuleUnavailable);
  let permissions;
  try {
    // Match the official @tauri-apps/plugin-geolocation guest binding: permission
    // state is queried through core.checkPermissions when the global Tauri API
    // exposes it. Keep the raw invoke only as compatibility fallback.
    permissions = typeof core.checkPermissions === 'function'
      ? await core.checkPermissions('geolocation')
      : await core.invoke('plugin:geolocation|check_permissions');
  } catch (error) {
    window.console?.warn?.('[virya:location] permission check failed', error);
    throw new Error(viryaTexts.locationModuleUnavailable);
  }

  let states = viryaLocationStates(permissions);
  const prompt = [states.precise, states.coarse].some(
    (state) => state === 'prompt' || state === 'prompt-with-rationale',
  );
  if (prompt) {
    try {
      permissions = await core.invoke('plugin:geolocation|request_permissions', {
        permissions: ['location'],
      });
      states = viryaLocationStates(permissions);
    } catch (error) {
      window.console?.warn?.('[virya:location] permission request failed', error);
      throw new Error(viryaTexts.locationDenied);
    }
  }

  if (states.precise === 'granted') return { precise: true };
  // Android 12+ lets users grant Approximate Location only. That is fully
  // sufficient for the "nearest AREA city" helper, while claim verification
  // still asks for precise/fresh samples and keeps the server-side accuracy gate.
  if (!preciseRequired && states.coarse === 'granted') return { precise: false };
  throw new Error(viryaTexts.locationDenied);
}

function viryaNormalizePosition(position) {
  // Keep compatibility with the official Position shape while accepting the
  // common flattened shape from older/generated Android bridges.
  const coords = position?.coords ?? position?.coordinates ?? position;
  const lat = Number(coords?.latitude ?? coords?.lat);
  const lng = Number(coords?.longitude ?? coords?.lng);
  const accuracy = Number(coords?.accuracy);
  const now = Date.now();
  const rawTimestamp = Number(position?.timestamp ?? position?.capturedAt);
  const epochTimestamp = Number.isFinite(rawTimestamp)
    ? (rawTimestamp < 10_000_000_000 ? rawTimestamp * 1000 : rawTimestamp)
    : now;
  // Android providers may return seconds, milliseconds, a monotonic value or
  // a stale cached timestamp. The AREA protocol needs epoch millis.
  const capturedAt = Math.round(
    Math.abs(epochTimestamp - now) <= 5 * 60_000 ? epochTimestamp : now,
  );
  if (!Number.isFinite(lat) || lat < -90 || lat > 90 ||
      !Number.isFinite(lng) || lng < -180 || lng > 180 ||
      !Number.isFinite(accuracy) || accuracy < 0 || accuracy > 10000 ||
      (Math.abs(lat) < 0.000001 && Math.abs(lng) < 0.000001)) {
    throw new Error(viryaTexts.locationUnavailable);
  }
  return { lat, lng, accuracy, capturedAt };
}

async function viryaPositionAttempt(core, options, deadlineMs) {
  const nativeRead = core.invoke('plugin:geolocation|get_current_position', { options });
  // The geolocation plugin documents that PositionOptions.timeout is ignored by
  // getCurrentPosition on Android, so enforce a UI-side deadline as well.
  let timer;
  try {
    const position = await Promise.race([
      nativeRead,
      new Promise((_, reject) => {
        timer = setTimeout(() => reject(new Error('location-read-timeout')), deadlineMs);
      }),
    ]);
    return viryaNormalizePosition(position);
  } finally {
    clearTimeout(timer);
    void nativeRead.catch(() => {});
  }
}

async function viryaWatchPositionAttempt(core, options, deadlineMs) {
  if (typeof core?.Channel !== 'function') {
    throw new Error('location-watch-channel-unavailable');
  }

  let settled = false;
  let resolveMessage;
  let rejectMessage;
  const result = new Promise((resolve, reject) => {
    resolveMessage = resolve;
    rejectMessage = reject;
  });
  const channel = new core.Channel((message) => {
    if (settled) return;
    if (typeof message === 'string') {
      settled = true;
      rejectMessage(new Error(message));
      return;
    }
    try {
      const normalized = viryaNormalizePosition(message);
      settled = true;
      resolveMessage(normalized);
    } catch (error) {
      window.console?.warn?.('[virya:location] discarded invalid watched position', error);
    }
  });

  let timer;
  try {
    await core.invoke('plugin:geolocation|watch_position', { options, channel });
    return await Promise.race([
      result,
      new Promise((_, reject) => {
        timer = setTimeout(() => reject(new Error('location-watch-timeout')), deadlineMs);
      }),
    ]);
  } finally {
    settled = true;
    clearTimeout(timer);
    if (Number.isFinite(channel.id)) {
      await core.invoke('plugin:geolocation|clear_watch', { channelId: channel.id }).catch((error) => {
        window.console?.warn?.('[virya:location] clear watch failed', error);
      });
    }
  }
}

async function viryaBrowserPositionAttempt(options, deadlineMs) {
  if (!window.navigator?.geolocation?.getCurrentPosition) {
    throw new Error('browser-geolocation-unavailable');
  }
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('browser-location-timeout')), deadlineMs);
    window.navigator.geolocation.getCurrentPosition(
      (position) => {
        clearTimeout(timer);
        try { resolve(viryaNormalizePosition(position)); }
        catch (error) { reject(error); }
      },
      (error) => {
        clearTimeout(timer);
        const code = Number(error?.code);
        const message = String(error?.message ?? error ?? 'browser-location-error');
        reject(new Error(`browser-location-error:${Number.isFinite(code) ? code : 'unknown'}:${message}`));
      },
      options,
    );
  });
}

async function viryaBrowserWatchPositionAttempt(options, deadlineMs) {
  const geolocation = window.navigator?.geolocation;
  if (!geolocation?.watchPosition || !geolocation?.clearWatch) {
    throw new Error('browser-geolocation-watch-unavailable');
  }

  return new Promise((resolve, reject) => {
    let settled = false;
    let watchId;
    let lastError;
    const finish = (fn, value) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      if (watchId !== undefined) {
        try { geolocation.clearWatch(watchId); } catch {}
      }
      fn(value);
    };
    const timer = setTimeout(() => {
      const detail = lastError ? `:${lastError}` : '';
      finish(reject, new Error(`browser-location-watch-timeout${detail}`));
    }, deadlineMs);

    watchId = geolocation.watchPosition(
      (position) => {
        try { finish(resolve, viryaNormalizePosition(position)); }
        catch (error) {
          lastError = String(error?.message ?? error);
          window.console?.warn?.('[virya:location] discarded invalid browser watched position', error);
        }
      },
      (error) => {
        const code = Number(error?.code);
        const message = String(error?.message ?? error ?? 'browser-location-watch-error');
        lastError = `${Number.isFinite(code) ? code : 'unknown'}:${message}`;
        // Permission denied is terminal. POSITION_UNAVAILABLE/TIMEOUT can be
        // transient while Android warms the provider, so keep the watch alive
        // until our own bounded deadline.
        if (code === 1) finish(reject, new Error(`browser-location-denied:${message}`));
      },
      options,
    );
  });
}

const viryaAndroidLocationPending = new Map();

window.__viryaAndroidLocationResult = (requestId, payload) => {
  const pending = viryaAndroidLocationPending.get(String(requestId));
  if (!pending) return;
  viryaAndroidLocationPending.delete(String(requestId));
  clearTimeout(pending.timer);
  if (payload?.ok === true) {
    try { pending.resolve(viryaNormalizePosition(payload)); }
    catch (error) { pending.reject(error); }
    return;
  }
  pending.reject(new Error(`android-location-error:${String(payload?.error ?? 'unknown')}`));
};

async function viryaAndroidLocationManagerAttempt(deadlineMs = 12000) {
  const bridge = window.ViryaAndroidLocation;
  if (!bridge || typeof bridge.requestLocation !== 'function') {
    throw new Error('android-location-bridge-unavailable');
  }

  const requestId = `area-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
  return await new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      viryaAndroidLocationPending.delete(requestId);
      reject(new Error('android-location-bridge-timeout'));
    }, deadlineMs);
    viryaAndroidLocationPending.set(requestId, { resolve, reject, timer });
    try {
      bridge.requestLocation(requestId);
    } catch (error) {
      clearTimeout(timer);
      viryaAndroidLocationPending.delete(requestId);
      reject(error);
    }
  });
}

async function viryaReadBrowserLocatorPosition() {
  let lastError;
  try {
    // AREA discovery only needs city-level accuracy. A cached/network position
    // is intentionally preferred indoors and mirrors the working web page path.
    return await viryaBrowserPositionAttempt(
      { enableHighAccuracy: false, timeout: 10000, maximumAge: 900000 },
      12000,
    );
  } catch (error) {
    lastError = error;
    window.console?.warn?.('[virya:location] browser locator current-position failed', error);
  }

  try {
    // Some Android WebViews return POSITION_UNAVAILABLE for one-shot reads while
    // the provider is waking up. Watching until the first valid fix avoids that
    // race without exposing exact coordinates to AREA claim verification.
    return await viryaBrowserWatchPositionAttempt(
      { enableHighAccuracy: false, timeout: 12000, maximumAge: 900000 },
      18000,
    );
  } catch (error) {
    lastError = error;
    window.console?.warn?.('[virya:location] browser locator watch failed', error);
  }

  throw lastError ?? new Error('browser-location-unavailable');
}

async function viryaReadCurrentPosition(core, strictFresh = false) {
  const directAttempts = strictFresh
    ? [
        [{ enableHighAccuracy: true, timeout: 12000, maximumAge: 0 }, 14000],
      ]
    : [
        // Prefer a recent cached/network fix first; it is plenty for selecting
        // the nearest city and avoids waiting for cold GPS indoors.
        [{ enableHighAccuracy: false, timeout: 4000, maximumAge: 300000 }, 5500],
        [{ enableHighAccuracy: true, timeout: 10000, maximumAge: 30000 }, 12000],
      ];

  let lastError;
  for (const [options, deadlineMs] of directAttempts) {
    try {
      return await viryaPositionAttempt(core, options, deadlineMs);
    } catch (error) {
      lastError = error;
      window.console?.warn?.('[virya:location] direct position attempt failed', options, error);
    }
  }

  // Android getCurrentPosition can return no fix while the provider is still
  // warming up. A one-shot watch waits for the next provider update and is the
  // reliable fallback recommended by the plugin API for live updates.
  const watchOptions = strictFresh
    ? { enableHighAccuracy: true, timeout: 15000, maximumAge: 0 }
    : { enableHighAccuracy: false, timeout: 12000, maximumAge: 300000 };
  try {
    return await viryaWatchPositionAttempt(core, watchOptions, strictFresh ? 18000 : 16000);
  } catch (error) {
    lastError = error;
    window.console?.warn?.('[virya:location] watched position attempt failed', error);
  }

  // Claim verification must never fall back to browser/WebView coordinates.
  // The server receives only fresh samples collected through the native plugin.
  if (strictFresh) {
    window.console?.warn?.('[virya:location] strict native position paths failed', lastError);
    throw new Error(viryaTexts.locationUnavailable);
  }

  window.console?.warn?.('[virya:location] native locator paths failed', lastError);
  throw new Error(viryaTexts.locationUnavailable);
}

export async function viryaCurrentPosition() {
  const core = await viryaWaitForNativeCore();
  // Ask for Android runtime permission through the official plugin, then use a
  // direct LocationManager bridge for AREA discovery. This avoids both Google
  // FusedLocationProvider and WebView POSITION_UNAVAILABLE failures observed on
  // real devices. Exact AREA claim verification does not use this bridge.
  await viryaEnsureLocationPermission(core, false);

  try {
    return await viryaAndroidLocationManagerAttempt(12000);
  } catch (androidError) {
    window.console?.warn?.('[virya:location] Android LocationManager locator failed; trying WebView', androidError);
  }

  try {
    return await viryaReadBrowserLocatorPosition();
  } catch (browserError) {
    window.console?.warn?.('[virya:location] WebView locator failed; trying Tauri plugin', browserError);
  }

  return viryaReadCurrentPosition(core, false);
}

export async function viryaCollectLocationSamples(minSamples, maxSamples, minDurationMs) {
  const minimum = Math.max(3, Math.min(Number(minSamples) || 3, 8));
  const maximum = Math.max(minimum, Math.min(Number(maxSamples) || 8, 8));
  const duration = Math.max(3000, Math.min(Number(minDurationMs) || 6000, 20000));
  const core = await viryaWaitForNativeCore();
  // Claim verification intentionally stays strict: precise permission and fresh
  // samples only. The relaxed fallback above is used solely for city discovery.
  await viryaEnsureLocationPermission(core, true);
  const startedAt = Date.now();
  const samples = [];
  let attempts = 0;
  const maxAttempts = maximum * 3;
  while (samples.length < maximum && attempts < maxAttempts) {
    attempts += 1;
    try {
      samples.push(await viryaReadCurrentPosition(core, true));
    } catch (error) {
      window.console?.warn?.('[virya:location] fresh claim sample failed', attempts, error);
    }
    const elapsed = Date.now() - startedAt;
    if (samples.length >= minimum && elapsed >= duration) break;
    await sleep(Math.min(1500, Math.max(700, duration - elapsed)));
  }
  if (samples.length < minimum || samples.at(-1).capturedAt - samples[0].capturedAt < duration) {
    throw new Error(viryaTexts.locationUnavailable);
  }
  return samples;
}


const VIRYA_FAILURE_STORAGE_KEY = 'virya:last-runtime-failure:v2';
const VIRYA_FAILURE_HISTORY_KEY = 'virya:runtime-failure-history:v3';
const MAX_RUNTIME_FAILURES = 8;

function viryaRuntimeMessage(error) {
  if (typeof error === 'string') return error;
  if (typeof error?.message === 'string') return error.message;
  try { return JSON.stringify(error); } catch { return viryaTexts.unknownError; }
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
    `${viryaTexts.reportType}: ${report.kind}`,
    `${viryaTexts.reportTime}: ${report.occurredAt}`,
    report.operation ? `${viryaTexts.reportOperation}: ${report.operation}` : '',
    report.path ? `${viryaTexts.reportPath}: ${report.path}` : '',
    `${viryaTexts.reportError}: ${report.message}`,
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
      <p class="eyebrow">${viryaTexts.diagnostics}</p>
      <h2>${previous ? viryaTexts.previousFailure : viryaTexts.currentFailure}</h2>
      <p>${viryaTexts.reportHelp}</p>
      <pre></pre>
      <div class="virya-runtime-failure-actions">
        <button type="button" data-action="copy">${viryaTexts.copyReport}</button>
        <button type="button" data-action="reload">${viryaTexts.restart}</button>
        <button type="button" data-action="close" class="ghost">${viryaTexts.close}</button>
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
      if (status) status.textContent = viryaTexts.reportCopied;
    } catch {
      if (status) status.textContent = viryaTexts.copyManually;
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
    viryaTemplate(viryaTexts.interrupted, 'command', operation.command),
  );
  viryaStorageRemove(VIRYA_OPERATION_STORAGE_KEY);
}

function viryaRecoverBootDiagnostic(report) {
  const diagnostic = window.__VIRYA_BOOT_DIAGNOSTIC__;
  if (!diagnostic) return;
  report(
    String(diagnostic.kind || 'unexpected-foreground-termination'),
    String(diagnostic.message || viryaTexts.uncleanShutdown),
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
    #[wasm_bindgen(js_name = viryaNativeBridgeAvailable)]
    fn native_bridge_available_js() -> bool;

    #[wasm_bindgen(js_name = viryaReferralCodeFromLocation)]
    fn referral_code_from_location_js() -> String;

    #[wasm_bindgen(js_name = viryaReadFanTab)]
    fn read_fan_tab_js() -> String;

    #[wasm_bindgen(js_name = viryaWriteFanTab)]
    fn write_fan_tab_js(value: &str);

    #[wasm_bindgen(js_name = viryaReadRootMode)]
    fn read_root_mode_js() -> String;

    #[wasm_bindgen(js_name = viryaWriteRootMode)]
    fn write_root_mode_js(value: &str);

    #[wasm_bindgen(catch, js_name = viryaShareText)]
    fn share_text_js(title: &str, text: &str, url: &str) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, js_name = viryaCopyText)]
    fn copy_text_js(text: &str) -> Result<js_sys::Promise, JsValue>;
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

    #[wasm_bindgen(catch, js_name = viryaScanAndConfirmFan)]
    async fn scan_and_confirm_fan_js() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = viryaScanAndConfirmBeacon)]
    async fn scan_and_confirm_beacon_js() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = viryaCurrentPosition)]
    async fn current_position_js() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = viryaCollectLocationSamples)]
    async fn collect_location_samples_js(
        min_samples: u32,
        max_samples: u32,
        min_duration_ms: u32,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = viryaInstallRuntimeGuards)]
    fn install_runtime_guards_js();

    #[wasm_bindgen(js_name = viryaSetRuntimeTranslations)]
    fn set_runtime_translations_js(value: JsValue);
}
