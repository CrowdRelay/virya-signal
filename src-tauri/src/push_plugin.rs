//! Android-only bridge to the staged Firebase Messaging plugin.
//!
//! The plugin never receives the fan session. It only exposes OS permission
//! state and the current FCM registration token; CrowdRelay registration is
//! performed by the Rust shell with the Stronghold-backed fan session.

#[cfg(target_os = "android")]
mod android {
    use serde::Deserialize;
    use tauri::{
        AppHandle, Manager, Runtime,
        plugin::{Builder, PluginHandle, TauriPlugin},
    };

    const PLUGIN_IDENTIFIER: &str = "music.virya.signal.push";
    const PLUGIN_CLASS: &str = "SignalPushPlugin";

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TokenResponse {
        token: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PermissionResponse {
        permission_state: String,
    }

    pub struct SignalPush<R: Runtime>(PluginHandle<R>);

    pub fn init<R: Runtime>() -> TauriPlugin<R> {
        Builder::new("signal-push")
            .setup(|app, api| {
                let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, PLUGIN_CLASS)?;
                app.manage(SignalPush(handle));
                Ok(())
            })
            .build()
    }

    fn handle<R: Runtime>(app: &AppHandle<R>) -> tauri::State<'_, SignalPush<R>> {
        app.state::<SignalPush<R>>()
    }

    pub fn token<R: Runtime>(app: &AppHandle<R>) -> Result<String, String> {
        let response = handle(app)
            .0
            .run_mobile_plugin::<TokenResponse>("getToken", ())
            .map_err(|error| error.to_string())?;
        let token = response.token.trim();
        if token.len() < 16 || token.len() > 4096 || !token.is_ascii() {
            return Err("invalid FCM token returned by Android".to_owned());
        }
        Ok(token.to_owned())
    }

    pub fn permission<R: Runtime>(app: &AppHandle<R>) -> Result<String, String> {
        handle(app)
            .0
            .run_mobile_plugin::<PermissionResponse>("checkPermissions", ())
            .map(|response| response.permission_state)
            .map_err(|error| error.to_string())
    }

    pub fn request_permission<R: Runtime>(app: &AppHandle<R>) -> Result<String, String> {
        handle(app)
            .0
            .run_mobile_plugin::<PermissionResponse>("requestPermissions", ())
            .map(|response| response.permission_state)
            .map_err(|error| error.to_string())
    }

    pub fn delete_token<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
        handle(app)
            .0
            .run_mobile_plugin::<serde_json::Value>("deleteToken", ())
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

#[cfg(target_os = "android")]
pub use android::*;
