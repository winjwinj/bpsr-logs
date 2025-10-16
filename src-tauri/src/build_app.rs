use tauri::generate_context;
use tauri::Builder as TauriBuilder;
use log::info;
use crate::stop_windivert;

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
