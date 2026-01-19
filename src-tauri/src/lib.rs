mod build_app;
mod live;
mod packets;
mod protocol;
#[cfg(target_os = "windows")]
mod service;
mod utils;

use crate::build_app::build;
use crate::live::bptimer_state::create_bptimer_enabled;
use crate::live::opcodes_models::EncounterMutex;
use crate::live::player_state::{PlayerCacheMutex, PlayerStateMutex};
use chrono::Utc;
use log::{info, warn};
use std::fs;

use tauri::Emitter;
use tauri::menu::MenuBuilder;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{LogicalPosition, LogicalSize, Manager, Position, Size, Window, WindowEvent};
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use tauri_plugin_svelte::ManagerExt;
use tauri_plugin_window_state::{AppHandleExt, StateFlags};
use tauri_specta::{Builder, collect_commands};

pub const WINDOW_LIVE_LABEL: &str = "live";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Ignored in production GH action
    if let Err(e) = dotenvy::dotenv() {
        info!("No .env file found or error loading .env: {e}");
    } else {
        info!("Loaded .env file successfully");
    }

    std::panic::set_hook(Box::new(|info| {
        info!("App crashed! Info: {info:?}");
        #[cfg(target_os = "windows")]
        {
            info!("Unloading and removing WinDivert...");
            service::stop_windivert();
        }
    }));

    let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            live::commands::get_header_info,
            live::commands::get_dps_player_window,
            live::commands::get_dps_skill_window,
            live::commands::get_dps_boss_only_player_window,
            live::commands::get_dps_boss_only_skill_window,
            live::commands::get_heal_player_window,
            live::commands::get_heal_skill_window,
            live::commands::reset_encounter,
            live::commands::toggle_pause_encounter,
            live::commands::hard_reset,
            live::commands::quit_app,
            live::commands::get_test_player_window,
            live::commands::get_test_skill_window,
            live::commands::set_bptimer_enabled,
            live::commands::extract_modules_from_local_player,
        ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    {
        use specta_typescript::{BigIntExportBehavior, Typescript};
        builder
            .export(
                Typescript::new().bigint(BigIntExportBehavior::Number),
                "../src/lib/bindings.ts",
            )
            .expect("Failed to export typescript bindings");

        // Add @ts-nocheck comment to suppress type checking for generated file
        let bindings_path = "../src/lib/bindings.ts";
        if let Ok(content) = fs::read_to_string(bindings_path) {
            if !content.starts_with("// @ts-nocheck") {
                let updated_content = format!("// @ts-nocheck\n{}", content);
                fs::write(bindings_path, updated_content)
                    .expect("Failed to add @ts-nocheck to bindings.ts");
            }
        }
    }

    let tauri_builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
            info!("starting app v{}", app.package_info().version);
            // Clean up any leftover WinDivert resources from previous runs
            #[cfg(target_os = "windows")]
            service::stop_windivert();

            // Check app updates
            // https://v2.tauri.app/plugin/updater/#checking-for-updates
            #[cfg(not(debug_assertions))] // <- Only check for updates on release builds
            {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    update(handle).await.unwrap();
                });
            }

            let app_handle = app.handle().clone();

            // Setup stuff
            cleanup_old_logs(&app_handle);
            setup_logs(&app_handle).expect("failed to setup logs");
            setup_tray(&app_handle).expect("failed to setup tray");
            setup_autostart(&app_handle);

            // Live Meter
            // https://v2.tauri.app/learn/splashscreen/#start-some-setup-tasks
            let is_bptimer_enabled = app.svelte().get_or::<bool>("integration", "bptimer", true);
            app.manage(create_bptimer_enabled(is_bptimer_enabled)); // setup bptimer enabled state
            app.manage(EncounterMutex::default()); // setup encounter state
            app.manage(PlayerStateMutex::default()); // setup player state
            app.manage(PlayerCacheMutex::default()); // setup player cache
            tauri::async_runtime::spawn(
                async move { live::live_main::start(app_handle.clone()).await },
            );
            Ok(())
        })
        .on_window_event(on_window_event_fn)
        .plugin(tauri_plugin_clipboard_manager::init()) // used to read/write to the clipboard
        .plugin(tauri_plugin_updater::Builder::new().build()) // used for auto updating the app
        .plugin(tauri_plugin_window_state::Builder::default().build()) // used to remember window size/position https://v2.tauri.app/plugin/window-state/
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {})) // used to enforce only 1 instance of the app https://v2.tauri.app/plugin/single-instance/
        .plugin(tauri_plugin_svelte::init()); // used for settings file

    build(tauri_builder)
        .expect("error while running tauri application")
        .run(|_app_handle, event| {
            // https://stackoverflow.com/questions/77856626/close-tauri-window-without-closing-the-entire-app
            if let tauri::RunEvent::ExitRequested { /* api, */ .. } = event {
                #[cfg(target_os = "windows")]
                {
                    info!("App is closing! Cleaning up WinDivert resources...");
                    service::stop_windivert();
                }
            }
        });
}

#[cfg(not(debug_assertions))]
async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    use tauri_plugin_updater::UpdaterExt;

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

fn cleanup_old_logs(app: &tauri::AppHandle) {
    let Ok(log_dir) = app.path().app_log_dir() else {
        return;
    };
    let Ok(entries) = fs::read_dir(&log_dir) else {
        return;
    };

    let current_version = app.package_info().version.to_string();
    let version_prefix = format!("bpsr-logs-v{current_version}-");

    let mut session_logs: Vec<_> = Vec::new();
    let mut other_logs_to_delete: Vec<_> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };

        let file_name = entry.file_name().to_string_lossy().to_string();

        // Match session log pattern: bpsr-logs-v{version}-{timestamp}.log
        if file_name.starts_with(&version_prefix) && file_name.ends_with(".log") {
            session_logs.push((path, modified));
        }
        // Delete anything else (old format logs, etc.)
        else {
            other_logs_to_delete.push(path);
        }
    }

    // Sort session logs by modified time (newest first)
    session_logs.sort_by(|a, b| b.1.cmp(&a.1));

    // Keep only the last 3 session log files (not including current), delete older ones
    for (path, _) in session_logs.into_iter().skip(3) {
        other_logs_to_delete.push(path);
    }

    // Delete all old/unwanted log files
    for path in other_logs_to_delete {
        if let Err(e) = fs::remove_file(&path) {
            eprintln!("Failed to delete old log file {path:?}: {e}");
        }
    }
}

fn setup_logs(app: &tauri::AppHandle) -> tauri::Result<()> {
    let app_version = &app.package_info().version;
    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let log_file_name = format!("bpsr-logs-v{app_version}-{timestamp}");

    app.plugin(
        tauri_plugin_log::Builder::new() // https://v2.tauri.app/plugin/logging/
            .clear_targets()
            .with_colors(ColoredLevelConfig::default())
            .targets([
                #[cfg(debug_assertions)]
                tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout)
                    .filter(|metadata| metadata.level() <= log::LevelFilter::Info),
                tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                    file_name: Some(log_file_name),
                })
                .filter(|metadata| metadata.level() <= log::LevelFilter::Info), // Exclude DEBUG logs from file
            ])
            .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
            .max_file_size(104857600)
            .build(),
    )?;
    Ok(())
}

fn setup_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    fn show_window(window: &tauri::WebviewWindow) -> tauri::Result<()> {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
        if window.label() == WINDOW_LIVE_LABEL {
            window.set_ignore_cursor_events(false)?;
        }
        Ok(())
    }

    let menu = MenuBuilder::new(app)
        .text("toggle-settings", "Toggle Settings")
        .text("show-live", "Show Live Meter")
        .text("reset", "Reset Window")
        .text("disable-clickthrough", "Disable Clickthrough")
        .separator()
        .text("quit", "Quit")
        .build()?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(|tray_app, event| match event.id.as_ref() {
            "toggle-settings" => {
                let tray_app_handle = tray_app.app_handle();
                let Some(live_meter_window) = tray_app_handle.get_webview_window(WINDOW_LIVE_LABEL)
                else {
                    return;
                };
                if let Err(e) = live_meter_window.emit("toggle-settings", ()) {
                    warn!("failed to emit toggle-settings event: {e}");
                }
            }
            "show-live" => {
                let tray_app_handle = tray_app.app_handle();
                let Some(live_meter_window) = tray_app_handle.get_webview_window(WINDOW_LIVE_LABEL)
                else {
                    return;
                };
                if let Err(e) = show_window(&live_meter_window) {
                    warn!("failed to show live meter window: {e}");
                }
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
                if let Err(e) = live_meter_window
                    .set_position(Position::Logical(LogicalPosition { x: 100.0, y: 100.0 }))
                {
                    warn!("failed to set default window position: {e}");
                }
                if let Err(e) = show_window(&live_meter_window) {
                    warn!("failed to show live meter window: {e}");
                }
            }
            "disable-clickthrough" => {
                let Some(live_meter_window) = tray_app.get_webview_window(WINDOW_LIVE_LABEL) else {
                    return;
                };
                if let Err(e) = live_meter_window.set_ignore_cursor_events(false) {
                    warn!("failed to disable clickthrough: {e}");
                }
            }
            "quit" => {
                #[cfg(target_os = "windows")]
                service::stop_windivert();
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
                if let Err(e) = show_window(&live_meter_window) {
                    warn!("failed to show live meter window: {e}");
                }
            }
        })
        .build(app)?;
    Ok(())
}

fn setup_autostart(app: &tauri::AppHandle) {
    use tauri_plugin_autostart::ManagerExt;

    let autostart_manager = app.autolaunch();
    if let Err(e) = if app.svelte().get_or::<bool>("general", "autostart", true) {
        autostart_manager.enable()
    } else {
        autostart_manager.disable()
    } {
        warn!("failed to set autostart: {e}");
    }
    match autostart_manager.is_enabled() {
        Ok(enabled) => info!("registered for autostart? {enabled}"),
        Err(e) => warn!("failed to check autostart status: {e}"),
    }
}

fn on_window_event_fn(window: &Window, event: &WindowEvent) {
    match event {
        // when you click the X button to close a window, don't close it - hide it!
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            if let Err(e) = window.hide() {
                warn!("failed to hide window: {e}");
            }
        }
        WindowEvent::Focused(focused) if !focused => {
            if let Err(e) = window.app_handle().save_window_state(StateFlags::all()) {
                warn!("failed to save window state to disk: {e}");
            }
        }
        _ => {}
    }
}
