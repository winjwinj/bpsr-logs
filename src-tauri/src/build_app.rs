use tauri::generate_context;
use tauri::Builder as TauriBuilder;
use log::info;
use crate::stop_windivert;

// https://discord.com/channels/616186924390023171/1400593249063927960/1400593249063927960
// RustRover + Tauri does not play nicely if this is not extracted into its own file.
pub fn build_and_run(builder: TauriBuilder<tauri::Wry>) {
    builder
        .build(generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                stop_windivert();
                info!("App is closing! Cleaning up resources...");
            }
        });
}
