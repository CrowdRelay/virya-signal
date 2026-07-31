use js_sys::Promise;
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = [window, __TAURI__, core], js_name = invoke)]
    fn tauri_invoke(command: &str, args: JsValue) -> Promise;
}

#[wasm_bindgen(inline_js = r#"
export async function viryaScanQr() {
  const scanner = window.__TAURI__?.barcodeScanner;
  if (!scanner?.scan) throw new Error('Skaner jest dostępny tylko w aplikacji iOS/Android.');
  const format = scanner.Format?.QRCode ?? 'QR_CODE';
  const result = await scanner.scan({ formats: [format] });
  if (typeof result === 'string') return result;
  return result?.content ?? result?.rawValue ?? result?.text ?? '';
}
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = viryaScanQr)]
    async fn scan_qr_js() -> Result<JsValue, JsValue>;
}

pub async fn invoke<T, A>(command: &str, args: &A) -> Result<T, String>
where
    T: DeserializeOwned,
    A: Serialize + ?Sized,
{
    let args = serde_wasm_bindgen::to_value(args).map_err(|error| error.to_string())?;
    let value = JsFuture::from(tauri_invoke(command, args))
        .await
        .map_err(js_error)?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub async fn invoke_unit<A>(command: &str, args: &A) -> Result<(), String>
where
    A: Serialize + ?Sized,
{
    let _: serde_json::Value = invoke(command, args).await?;
    Ok(())
}

pub async fn scan_qr() -> Result<String, String> {
    let value = scan_qr_js().await.map_err(js_error)?;
    value
        .as_string()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Skaner nie zwrócił kodu.".to_owned())
}

fn js_error(value: JsValue) -> String {
    value
        .as_string()
        .or_else(|| js_sys::JSON::stringify(&value).ok().and_then(|v| v.as_string()))
        .unwrap_or_else(|| "Nieznany błąd aplikacji".to_owned())
}
