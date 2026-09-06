#[wasm_bindgen(inline_js = r#"
const VIRYA_FAN_TAB_STORAGE_KEY = 'virya:fan-tab:v1';
const VIRYA_ROOT_MODE_STORAGE_KEY = 'virya:root-mode:v1';
const VIRYA_TRANSIENT_ROOT_MODE_STORAGE_KEY = 'virya:root-mode-transient:v1';

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
    if (window.sessionStorage?.getItem(VIRYA_TRANSIENT_ROOT_MODE_STORAGE_KEY) === 'team') {
      return 'team';
    }
    const value = String(window.localStorage?.getItem(VIRYA_ROOT_MODE_STORAGE_KEY) ?? 'fan');
    return value === 'latarnik' ? 'latarnik' : 'fan';
  } catch {
    return 'fan';
  }
}

export function viryaWriteRootMode(value) {
  const requested = String(value);
  try {
    if (requested === 'team') {
      window.sessionStorage?.setItem(VIRYA_TRANSIENT_ROOT_MODE_STORAGE_KEY, 'team');
      return;
    }
    window.sessionStorage?.removeItem(VIRYA_TRANSIENT_ROOT_MODE_STORAGE_KEY);
    const safe = requested === 'latarnik' ? 'latarnik' : 'fan';
    window.localStorage?.setItem(VIRYA_ROOT_MODE_STORAGE_KEY, safe);
  } catch {}
}

// Android's back gesture is handled by the WebView: tao's activity calls
// `goBack()` when there is history to go back to, and finishes the activity
// otherwise. With a single-document app there is never any history, so back
// closed the app from the middle of checkout. One guard entry is pushed while
// any dismissible layer is open; back consumes it, the app closes that layer,
// and a still-open layer below pushes the next guard.
//
// Android WebView (especially the first pushState after page load) can fire a
// spurious popstate synchronously after pushState. Without suppression, the
// back handler fires immediately, closes the layer that was just opened, and
// the tab snaps back to Signal on the first tap. The flag is cleared on the
// next macrotask — a real back gesture arrives as a later event, never
// synchronously off the pushState call itself.
let viryaSuppressPopstate = false;
export function viryaPushBackGuard() {
  try {
    viryaSuppressPopstate = true;
    window.history?.pushState({ virya: 'back-guard' }, '');
    setTimeout(() => { viryaSuppressPopstate = false; }, 0);
  } catch {}
}

const VIRYA_PUSH_PRIMER_KEY = 'virya:push-primer:v1';

// Whether the fan has already been asked, in our own words, to turn
// notifications on. Android 13 grants exactly one POST_NOTIFICATIONS dialog: a
// denial there is permanent short of a trip to system settings, so the ask has
// to be spent deliberately and never repeated on every launch.
export function viryaPushPrimerSeen() {
  try {
    return window.localStorage?.getItem(VIRYA_PUSH_PRIMER_KEY) === 'seen';
  } catch {
    // No storage means no memory of asking, and asking twice is worse than
    // never asking: treat it as already spent.
    return true;
  }
}

export function viryaMarkPushPrimerSeen() {
  try { window.localStorage?.setItem(VIRYA_PUSH_PRIMER_KEY, 'seen'); } catch {}
}

export function viryaGoBack() {
  try { window.history?.back(); } catch {}
}

export function viryaInstallBackHandler(callback) {
  const handler = () => {
    if (viryaSuppressPopstate) return;
    try { callback(); } catch {}
  };
  try { window.addEventListener('popstate', handler); } catch { return () => {}; }
  return () => { try { window.removeEventListener('popstate', handler); } catch {} };
}

const VIRYA_UPDATE_DISMISSED_KEY = 'virya:update-dismissed:v1';

// The update banner is dismissed per-version: storing the version string
// means a new release re-triggers the banner, while the current one stays
// hidden. No storage means no dismissal memory, which is treated as
// "not dismissed" so the banner can still appear.
export function viryaReadDismissedUpdate() {
  try { return window.localStorage?.getItem(VIRYA_UPDATE_DISMISSED_KEY) || ''; } catch { return ''; }
}

export function viryaWriteDismissedUpdate(version) {
  try { window.localStorage?.setItem(VIRYA_UPDATE_DISMISSED_KEY, String(version || '')); } catch {}
}

"#)]
extern "C" {
    #[wasm_bindgen(js_name = viryaReadFanTab)]
    fn read_fan_tab_js() -> String;

    #[wasm_bindgen(js_name = viryaWriteFanTab)]
    fn write_fan_tab_js(value: &str);

    #[wasm_bindgen(js_name = viryaReadRootMode)]
    fn read_root_mode_js() -> String;

    #[wasm_bindgen(js_name = viryaWriteRootMode)]
    fn write_root_mode_js(value: &str);

    #[wasm_bindgen(js_name = viryaPushBackGuard)]
    fn push_back_guard_js();

    #[wasm_bindgen(js_name = viryaPushPrimerSeen)]
    fn push_primer_seen_js() -> bool;

    #[wasm_bindgen(js_name = viryaMarkPushPrimerSeen)]
    fn mark_push_primer_seen_js();

    #[wasm_bindgen(js_name = viryaGoBack)]
    fn go_back_js();

    #[wasm_bindgen(js_name = viryaInstallBackHandler)]
    fn install_back_handler_js(callback: &js_sys::Function) -> js_sys::Function;

    #[wasm_bindgen(js_name = viryaReadDismissedUpdate)]
    fn read_dismissed_update_js() -> String;

    #[wasm_bindgen(js_name = viryaWriteDismissedUpdate)]
    fn write_dismissed_update_js(version: &str);
}
