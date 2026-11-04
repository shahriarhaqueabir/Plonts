use js_sys::Promise;
use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = "invoke")]
    fn invoke_raw(cmd: &str, args: JsValue) -> Promise;
}

/// Invoke a Tauri command with serialized args, returning a typed result.
pub async fn call<T: DeserializeOwned, A: Serialize>(cmd: &str, args: &A) -> Result<T, String> {
    let args_js = serde_wasm_bindgen::to_value(args).map_err(|e| e.to_string())?;
    let promise = invoke_raw(cmd, args_js);
    let js = JsFuture::from(promise)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "invoke failed".to_string()))?;
    serde_wasm_bindgen::from_value(js).map_err(|e| e.to_string())
}

/// Invoke a Tauri command that takes no arguments.
pub async fn call0<T: DeserializeOwned>(cmd: &str) -> Result<T, String> {
    let promise = invoke_raw(cmd, JsValue::UNDEFINED);
    let js = JsFuture::from(promise)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "invoke failed".to_string()))?;
    serde_wasm_bindgen::from_value(js).map_err(|e| e.to_string())
}