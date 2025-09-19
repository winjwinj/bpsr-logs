mod live;
mod packets;

use crate::live::opcodes_models::EncounterMutex;
use log::info;
use tauri::Manager;
use tauri_plugin_log::fern::colors::ColoredLevelConfig;

use specta_typescript::{BigIntExportBehavior, Typescript};
use tauri_specta::{collect_commands, Builder};
use window_vibrancy::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let time_now = Some(format!(
        "{:?}",
        chrono::Utc::now()
            .timestamp_nanos_opt()
            .expect("time out of bounds")
    ));

    let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![live::commands::get_damage_row,]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(
            Typescript::new().bigint(BigIntExportBehavior::BigInt),
            "../src/lib/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
            info!("starting app v{}", app.package_info().version);

            // Setup Blur
            let window = app.get_webview_window("live").unwrap();
            #[cfg(target_os = "windows")]
            apply_blur(&window, Some((18, 18, 18, 125))).expect("Unsupported platform! 'apply_blur' is only supported on Windows");

            app.manage(EncounterMutex::default()); // todo: maybe use https://github.com/ferreira-tb/tauri-store

            // TODO: Setup auto updater

            // Live Meter
            // https://v2.tauri.app/learn/splashscreen/#start-some-setup-tasks
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(
                async move { live::live_main::start(app_handle.clone()).await },
            );
            Ok(())
        })
        .plugin(tauri_plugin_window_state::Builder::default().build()) // used to remember window size/position https://v2.tauri.app/plugin/window-state/
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {})) // used to enforce only 1 instance of the app https://v2.tauri.app/plugin/single-instance/
        .plugin(
            tauri_plugin_log::Builder::new() // https://v2.tauri.app/plugin/logging/
                .clear_targets()
                .with_colors(ColoredLevelConfig::default())
                .targets([
                    #[cfg(debug_assertions)]
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout)
                        .filter(|metadata| metadata.level() <= log::LevelFilter::Info),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: time_now,
                    }).filter(|metadata| metadata.level() <= log::LevelFilter::Info), // todo: remove info filter
                ])
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
