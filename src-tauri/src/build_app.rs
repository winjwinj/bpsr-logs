use tauri::Builder as TauriBuilder;
use tauri::{App, generate_context};

// https://discord.com/channels/616186924390023171/1400593249063927960/1400593249063927960
// RustRover + Tauri does not play nicely if this is not extracted into its own file.
pub fn build(builder: TauriBuilder<tauri::Wry>) -> tauri::Result<App> {
    builder.build(generate_context!())
}
