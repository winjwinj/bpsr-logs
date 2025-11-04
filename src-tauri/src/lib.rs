mod build_app;
mod live;
mod packets;
pub mod db;

use crate::build_app::build;
use crate::live::opcodes_models::EncounterMutex;
use log::{info, warn};
use std::process::Command;

use crate::live::commands::{disable_blur, enable_blur};
use tauri::menu::MenuBuilder;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{LogicalPosition, LogicalSize, Manager, Position, Size, Window, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use tauri_plugin_svelte::ManagerExt;
use tauri_plugin_window_state::{AppHandleExt, StateFlags};
use tauri_specta::{collect_commands, Builder};

pub const WINDOW_LIVE_LABEL: &str = "live";
pub const WINDOW_MAIN_LABEL: &str = "main";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::panic::set_hook(Box::new(|info| {
        info!("App crashed! Info: {info:?}");
        info!("Unloading and removing windivert...");
        stop_windivert();
        remove_windivert();
    }));

    let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            live::commands::enable_blur,
            live::commands::disable_blur,
            live::commands::copy_sync_container_data,
            live::commands::get_header_info,
            live::commands::get_dps_player_window,
            live::commands::get_dps_skill_window,
            live::commands::get_dps_boss_only_player_window,
            live::commands::get_dps_boss_only_skill_window,
            live::commands::get_heal_player_window,
            live::commands::get_heal_skill_window,
            live::commands::get_player_metadata,
            live::commands::update_player_metadata,
            live::commands::persist_player_metadata,
            live::commands::set_local_player_uid,
            live::commands::get_local_player_uid,
            live::commands::reset_encounter,
            live::commands::toggle_pause_encounter,
            live::commands::hard_reset,
            live::commands::get_test_player_window,
            live::commands::get_test_skill_window,
            live::history_commands::get_all_encounter_history,
            live::history_commands::get_encounter_detail,
            live::history_commands::delete_encounter_history,
            live::history_commands::clear_all_data,
            live::history_commands::get_historical_players_window,
            live::history_commands::get_historical_skills_window,
        ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    {
        use specta_typescript::{BigIntExportBehavior, Typescript};
        builder.export(Typescript::new().bigint(BigIntExportBehavior::Number), "../src/lib/bindings.ts")
               .expect("Failed to export typescript bindings");
    }

    let tauri_builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
            info!("starting app v{}", app.package_info().version);
            stop_windivert();
            remove_windivert();

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
            setup_logs(&app_handle).expect("failed to setup logs");
            setup_tray(&app_handle).expect("failed to setup tray");
            setup_autostart(&app_handle);
            setup_blur(&app_handle);

            // Setup database - tauri_plugin_sql handles connection pooling and migrations
            // The plugin will run migrations automatically
            let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
            let db_path = app_data_dir.join("encounters.db");
            
            // Ensure database file can be created by touching it if it doesn't exist
            if !db_path.exists() {
                match std::fs::File::create(&db_path) {
                    Ok(_) => {
                        log::info!("Created new database file at: {}", db_path.display());
                    }
                    Err(e) => {
                        log::error!("Failed to create database file: {}", e);
                        return Err(format!("Failed to create database file at {}: {}", db_path.display(), e).into());
                    }
                }
            }
            
            // Create connection pool using sqlx directly
            // SQLite URL format: sqlite:path/to/db.db (or sqlite::memory: for in-memory)
            let db_url = format!("sqlite:{}", db_path.to_string_lossy());
            log::info!("Initializing database at: {}", db_path.display());
            
            let db_connection = match tauri::async_runtime::block_on(async {
                match sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(20)
                    .connect(&db_url)
                    .await
                {
                    Ok(pool) => {
                        // Run migrations manually since we're managing the pool directly
                        for migration in db::init_db() {
                            match migration.kind {
                                tauri_plugin_sql::MigrationKind::Up => {
                                    log::info!("Running migration: v{} - {}", migration.version, migration.description);
                                    // Split migrations that might have multiple statements
                                    for statement in migration.sql.split(';').filter(|s| !s.trim().is_empty()) {
                                        match sqlx::query(statement).execute(&pool).await {
                                            Ok(_) => {
                                                log::debug!("Migration statement executed successfully");
                                            }
                                            Err(e) => {
                                                // Ignore errors for ALTER TABLE if column already exists
                                                if statement.contains("ALTER TABLE") && e.to_string().contains("duplicate") {
                                                    log::info!("Column already exists (skipped): {}", e);
                                                } else {
                                                    log::warn!("Migration warning: {}", e);
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {} // Skip Down migrations
                            }
                        }
                        
                        log::info!("Database initialized successfully");
                        Ok(pool)
                    }
                    Err(e) => {
                        log::error!("Failed to create database pool: {}", e);
                        Err(e)
                    }
                }
            }) {
                Ok(pool) => pool,
                Err(e) => {
                    log::error!("Database initialization failed: {}", e);
                    return Err(format!("Database initialization failed: {}", e).into());
                }
            };
            
            app.manage(db_connection);

            // Live Meter
            // https://v2.tauri.app/learn/splashscreen/#start-some-setup-tasks
            app.manage(EncounterMutex::default()); // setup encounter state
            tauri::async_runtime::spawn(
                async move { live::live_main::start(app_handle.clone()).await },
            );
            Ok(())
        })
        .on_window_event(on_window_event_fn)
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![])))
        .plugin(tauri_plugin_clipboard_manager::init()) // used to read/write to the clipboard
        .plugin(tauri_plugin_updater::Builder::new().build()) // used for auto updating the app
        .plugin(tauri_plugin_window_state::Builder::default().build()) // used to remember window size/position https://v2.tauri.app/plugin/window-state/
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {})) // used to enforce only 1 instance of the app https://v2.tauri.app/plugin/single-instance/
        .plugin(tauri_plugin_svelte::init()); // used for settings file

    build(tauri_builder).expect("error while running tauri application")
                        .run(|_app_handle, event| {
                            // https://stackoverflow.com/questions/77856626/close-tauri-window-without-closing-the-entire-app
                            if let tauri::RunEvent::ExitRequested { /* api, */ .. } = event {
                                stop_windivert();
                                info!("App is closing! Cleaning up resources...");
                            }
                        });
}

#[allow(unused)]
fn start_windivert() {
    let status = Command::new("sc")
        .args([
            "create",
            "windivert",
            "type=",
            "kernel",
            "binPath=",
            "WinDivert64.sys",
            "start=",
            "demand",
        ])
        .status();
    if status.is_ok_and(|status| status.success()) {
        info!("started driver");
    } else {
        warn!("could not execute command to stop driver");
    }
}

fn stop_windivert() {
    let status = Command::new("sc").args(["stop", "windivert"]).status();
    if status.is_ok_and(|status| status.success()) {
        info!("stopped driver");
    } else {
        warn!("could not execute command to stop driver");
    }
}

fn remove_windivert() {
    let status = Command::new("sc")
        .args(["delete", "windivert", "start=", "demand"])
        .status();
    if status.is_ok_and(|status| status.success()) {
        info!("deleted driver");
    } else {
        warn!("could not execute command to delete driver");
    }
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

fn setup_logs(app: &tauri::AppHandle) -> tauri::Result<()> {
    let app_version = &app.package_info().version;
    let pst_time = chrono::Utc::now()
        .with_timezone(&chrono_tz::America::Los_Angeles)
        .format("%m-%d-%Y %H_%M_%S")
        .to_string();
    let log_file_name = format!("log v{app_version} {pst_time} PST", );

    app.plugin(tauri_plugin_log::Builder::new() // https://v2.tauri.app/plugin/logging/
                   .clear_targets()
                   .with_colors(ColoredLevelConfig::default())
                   .targets([
                       #[cfg(debug_assertions)]
                       tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout)
                           .filter(|metadata| metadata.level() <= log::LevelFilter::Info),
                       tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                           file_name: Some(log_file_name),
                       }),
                   ])
                   .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                   .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(10)) // keep the last 10 logs
                   .max_file_size(1_073_741_824 /* 1 gb */)
                   .build())?;
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
        .text("show-settings", "Show Settings")
        .separator()
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
            "show-settings" => {
                let tray_app_handle = tray_app.app_handle();
                let Some(main_meter_window) = tray_app_handle.get_webview_window(WINDOW_MAIN_LABEL)
                else {
                    return;
                };
                if let Err(e) = show_window(&main_meter_window) {
                    warn!("failed to show main meter window: {e}");
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
                if let Err(e) = live_meter_window.set_position(Position::Logical(LogicalPosition { x: 100.0, y: 100.0 })) {
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
                stop_windivert();
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
                    warn!("failed to show main meter window: {e}");
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

fn setup_blur(app: &tauri::AppHandle) {
    if app.svelte().get_or::<bool>("accessibility", "blur", true) {
        enable_blur(app.clone());
    } else {
        disable_blur(app.clone());
    }
}

fn on_window_event_fn(window: &Window, event: &WindowEvent) {
    match event {
        // when you click the X button to close a window, don't close it - hide it!
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            if window.label() == WINDOW_MAIN_LABEL {
                if let Err(e) = window.hide() {
                    warn!("failed to hide main meter window: {e}");
                }
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
