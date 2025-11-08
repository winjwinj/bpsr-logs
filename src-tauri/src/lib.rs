mod build_app;
mod live;
mod packets;

use crate::build_app::build;
use crate::live::opcodes_models::{Encounter, EncounterMutex};
use crate::live::crowdsource_persistence::{apply_snapshot_to_encounter, load_snapshot};
use crate::live::bptimer_stream::{MobHpStore, MobHpStoreMutex, stream_control_channel};
use std::sync::Arc;
use parking_lot::RwLock;
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
            live::commands::get_last_hit_boss_name,
            live::commands::get_crowdsourced_monster,
            live::commands::get_crowdsourced_monster_options,
            live::commands::get_crowdsourced_mob_hp,
            live::commands::set_bptimer_stream_active,
            live::commands::set_crowdsourced_monster_remote,
            live::commands::get_local_player_line,
            live::commands::mark_current_crowdsourced_line_dead,
            live::commands::get_dps_player_window,
            live::commands::get_dps_skill_window,
            live::commands::get_dps_boss_only_player_window,
            live::commands::get_dps_boss_only_skill_window,
            live::commands::get_heal_player_window,
            live::commands::get_heal_skill_window,
            live::commands::reset_encounter,
            live::commands::toggle_pause_encounter,
            live::commands::hard_reset,
            live::commands::get_test_player_window,
            live::commands::get_test_skill_window,
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

            // Live Meter
            // https://v2.tauri.app/learn/splashscreen/#start-some-setup-tasks
            let mut encounter = Encounter::default();
            if let Some(snapshot) = load_snapshot(&app_handle) {
                apply_snapshot_to_encounter(&snapshot, &mut encounter);
            }
            let encounter_mutex: EncounterMutex = std::sync::Mutex::new(encounter);
            app.manage(encounter_mutex); // setup encounter state
            
            // BPTimer HP Store
            let mob_hp_store: MobHpStoreMutex = Arc::new(RwLock::new(MobHpStore::default()));
            app.manage(mob_hp_store.clone());
            let (stream_control_sender, stream_control_receiver) =
                stream_control_channel();
            app.manage(stream_control_sender.clone());
            
            tauri::async_runtime::spawn(
                async move { live::live_main::start(app_handle.clone()).await },
            );
            
            // BPTimer SSE Stream
            let app_handle_stream = app.handle().clone();
            let stream_store = mob_hp_store.clone();
            tauri::async_runtime::spawn(
                async move {
                    live::bptimer_stream::start_bptimer_stream(
                        app_handle_stream,
                        stream_store,
                        stream_control_receiver,
                    )
                    .await
                },
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
