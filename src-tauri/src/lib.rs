mod live;
mod packets;

use crate::live::opcodes_models::EncounterMutex;
use log::{info, warn};
use specta_typescript::{BigIntExportBehavior, Typescript};
use std::process::Command;
use window_vibrancy::apply_blur;

use tauri::menu::{Menu, MenuBuilder, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{LogicalPosition, LogicalSize, Manager, Position, Size, Window, WindowEvent};
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_window_state::{AppHandleExt, StateFlags};
use tauri_specta::{Builder, collect_commands};

pub const WINDOW_LIVE_LABEL: &str = "live";
pub const WINDOW_MAIN_LABEL: &str = "main";

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
        .commands(collect_commands![
            live::commands::get_header_info,
            live::commands::get_damage_window,
            live::commands::get_skill_window,
            live::commands::get_heal_skill_window,
            live::commands::get_heal_window,
            live::commands::reset_encounter,
            live::commands::toggle_pause_encounter,
        ]);

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

            // Check app updates
            // https://v2.tauri.app/plugin/updater/#checking-for-updates
            #[cfg(not(debug_assertions))] // <- Only check for updates on release builds
            {
                unload_and_remove_driver();

                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    crate::update(handle).await.unwrap();
                });
            }

            let app_handle = app.handle().clone();

            // Setup tray icon
            setup_tray(&app_handle).expect("failed to setup tray");

            // Setup blur
            setup_blur(&app_handle);

            app.manage(EncounterMutex::default());

            // Live Meter
            // https://v2.tauri.app/learn/splashscreen/#start-some-setup-tasks
            tauri::async_runtime::spawn(
                async move { live::live_main::start(app_handle.clone()).await },
            );
            Ok(())
        })
        .on_window_event(on_window_event)
        .plugin(tauri_plugin_clipboard_manager::init()) // used to read/write to the clipboard
        .plugin(tauri_plugin_updater::Builder::new().build()) // used for auto updating the app
        .plugin(tauri_plugin_window_state::Builder::default().build()) // used to remember window size/position https://v2.tauri.app/plugin/window-state/
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {})) // used to enforce only 1 instance of the app https://v2.tauri.app/plugin/single-instance/
        .plugin(tauri_plugin_svelte::init()) // used for settings file
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
                    })
                    .filter(|metadata| metadata.level() <= log::LevelFilter::Info), // todo: remove info filter
                ])
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn unload_and_remove_driver() {
    let status = Command::new("sc").args(["stop", "windivert"]).status();
    if status.is_ok_and(|status| status.success()) {
        info!("stopped driver");
    } else {
        warn!("could not execute command to stop driver");
    }

    let status = Command::new("sc").args(["delete", "windivert"]).status();
    status.expect("unable to delete driver");
}

async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
        let mut downloaded = 0;
        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    info!("downloaded {downloaded} from {content_length:?}");
                },
                || {
                    info!("download finished");
                },
            )
            .await?;

        info!("update installed");
        app.restart();
    }
    Ok(())
}

fn setup_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    fn show_window(window: &tauri::WebviewWindow) {
        window.show().unwrap();
        window.unminimize().unwrap();
        window.set_focus().unwrap();
        if window.label() == WINDOW_LIVE_LABEL {
            window.set_ignore_cursor_events(false).unwrap();
        }
    }

    let menu = MenuBuilder::new(app)
        .text("show-settings", "Show Settings")
        .separator()
        .text("show-live", "Show Live Meter")
        .separator()
        .text("reset", "Reset Window")
        .separator()
        .text("quit", "Quit")
        .build()?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(|tray_app, event| match event.id.as_ref() {
            "show-settings" => {
                let tray_app_handle = tray_app.app_handle();
                let Some(main_meter_window) = tray_app_handle.get_webview_window(WINDOW_MAIN_LABEL)
                else {
                    return;
                };
                show_window(&main_meter_window);
            }
            "show-live" => {
                let tray_app_handle = tray_app.app_handle();
                let Some(live_meter_window) = tray_app_handle.get_webview_window(WINDOW_LIVE_LABEL)
                else {
                    return;
                };
                show_window(&live_meter_window);
            }
            "reset" => {
                let Some(live_meter_window) = tray_app.get_webview_window(WINDOW_LIVE_LABEL) else {
                    return;
                };
                live_meter_window
                    .set_size(Size::Logical(LogicalSize {
                        width: 500.0,
                        height: 350.0,
                    }))
                    .unwrap();
                live_meter_window
                    .set_position(Position::Logical(LogicalPosition { x: 100.0, y: 100.0 }))
                    .unwrap();
                live_meter_window.show().unwrap();
                live_meter_window.unminimize().unwrap();
                live_meter_window.set_focus().unwrap();
                live_meter_window.set_ignore_cursor_events(false).unwrap();
            }
            "quit" => {
                tray_app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                // Show and focus the live meter window when the tray is clicked
                let app = tray.app_handle();
                let Some(live_meter_window) = app.get_webview_window(WINDOW_LIVE_LABEL) else {
                    return;
                };
                show_window(&live_meter_window);
            }
        })
        .build(app)?;
    Ok(())
}

fn setup_blur(app: &tauri::AppHandle) {
    if let Some(meter_window) = app.get_webview_window(WINDOW_LIVE_LABEL) {
        apply_blur(&meter_window, Some((10, 10, 10, 50))).ok();
    }
}

fn on_window_event(window: &Window, event: &WindowEvent) {
    match event {
        // when you click the X button to close a window
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close(); // don't close it, just hide it
            if window.label() == WINDOW_MAIN_LABEL {
                window.hide().unwrap();
            }
        }
        WindowEvent::Focused(focused) if !focused => {
            window.app_handle().save_window_state(StateFlags::all());
        }
        _ => {}
    }
}
