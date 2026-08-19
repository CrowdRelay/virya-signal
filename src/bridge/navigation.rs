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
}
