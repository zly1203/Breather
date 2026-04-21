mod state;
mod rules;
mod messages;
mod server;
mod hook_install;

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use tauri::{
    menu::{Menu, MenuItem},
    Emitter, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder,
};

use state::StateManager;

const WINDOW_LABEL: &str = "main";
const WINDOW_WIDTH: f64 = 320.0;
const WINDOW_HEIGHT: f64 = 420.0;

/// Debounce guard: timestamp (ms) of the last toggle.
/// Prevents double-toggle from press+release firing two Click events.
static LAST_TOGGLE_MS: AtomicI64 = AtomicI64::new(0);

#[tauri::command]
fn get_session_stats(state_mgr: tauri::State<'_, Arc<StateManager>>) -> serde_json::Value {
    let character = rules::character_state(&state_mgr);
    match state_mgr.get_stats() {
        Some(stats) => {
            let mut v = serde_json::to_value(stats).unwrap_or_default();
            v["character"] = serde_json::json!(character);
            v
        }
        None => serde_json::json!({"active": false, "character": character}),
    }
}

#[tauri::command]
fn set_intensity(state_mgr: tauri::State<'_, Arc<StateManager>>, level: u8) {
    state_mgr.set_intensity(level);
    state_mgr.save();
}

#[tauri::command]
fn get_intensity(state_mgr: tauri::State<'_, Arc<StateManager>>) -> u8 {
    state_mgr.state.lock().unwrap().intensity
}

#[tauri::command]
fn get_history(state_mgr: tauri::State<'_, Arc<StateManager>>) -> serde_json::Value {
    serde_json::to_value(state_mgr.get_history()).unwrap_or_default()
}

#[tauri::command]
fn get_today_total(state_mgr: tauri::State<'_, Arc<StateManager>>) -> u32 {
    state_mgr.get_today_total()
}

#[tauri::command]
fn get_today_history(state_mgr: tauri::State<'_, Arc<StateManager>>) -> serde_json::Value {
    serde_json::to_value(state_mgr.get_today_history()).unwrap_or_default()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state_mgr = Arc::new(StateManager::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(state_mgr.clone())
        .invoke_handler(tauri::generate_handler![
            get_session_stats,
            set_intensity,
            get_intensity,
            get_history,
            get_today_total,
            get_today_history,
        ])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                server::init_macos_notifications(&app.config().identifier);
            }

            // Pre-create the window hidden so the first click is instant
            // (webview, HTML, CSS, fonts are all loaded before the user asks).
            create_window(&app.handle().clone(), false);

            // Connect to Claude Code: install hook.sh and register it in
            // settings.json (idempotent, safe to run every launch).
            let handle_for_hook = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Run on a blocking thread so file I/O doesn't stall the runtime.
                let outcome = tokio::task::spawn_blocking(hook_install::ensure_hook_installed)
                    .await
                    .unwrap_or(hook_install::InstallOutcome::Error("join failed".into()));
                match outcome {
                    hook_install::InstallOutcome::Installed => {
                        server::send_notification(
                            &handle_for_hook,
                            "Breather",
                            "All set. I'll be listening in the background.",
                        );
                    }
                    hook_install::InstallOutcome::ClaudeNotFound => {
                        log::info!("Claude Code not detected; hook not installed");
                    }
                    hook_install::InstallOutcome::AlreadyInstalled => {
                        log::info!("hook already registered");
                    }
                    hook_install::InstallOutcome::Error(e) => {
                        log::warn!("hook install failed: {}", e);
                    }
                }
            });

            // Right-click menu: just Quit.
            let quit = MenuItem::with_id(app, "quit", "Quit Breather", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit])?;

            let tray = app.tray_by_id("main").expect("tray icon not found");
            tray.set_menu(Some(menu))?;
            tray.set_show_menu_on_left_click(false)?;

            tray.on_menu_event(|app, event| {
                if event.id.as_ref() == "quit" {
                    if let Some(mgr) = app.try_state::<Arc<StateManager>>() {
                        mgr.close_current_session();
                        mgr.save();
                    }
                    app.exit(0);
                }
            });

            // Left-click: toggle window. Debounced at 400ms to absorb
            // the Down+Up pair that macOS fires for a single click.
            tray.on_tray_icon_event(|tray, event| {
                if let tauri::tray::TrayIconEvent::Click {
                    button: tauri::tray::MouseButton::Left, ..
                } = event {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as i64;
                    let prev = LAST_TOGGLE_MS.load(Ordering::Relaxed);
                    if now - prev > 400 {
                        LAST_TOGGLE_MS.store(now, Ordering::Relaxed);
                        toggle_window(tray.app_handle());
                    }
                }
            });

            // HTTP server for hook events.
            let mgr = state_mgr.clone();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                server::start_server(mgr, handle).await;
            });

            // Background timer: check for stale sessions every 2 minutes.
            let mgr2 = state_mgr.clone();
            let handle2 = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(120)).await;
                    if let Some(summary) = mgr2.check_and_close_stale() {
                        mgr2.save();
                        handle2.emit("session-updated", ()).ok();
                        if summary.duration_minutes >= 5 && summary.interaction_count >= 5 {
                            let msg = format!(
                                "Session complete. {}, {} interactions. Nice work, go get some air.",
                                server::format_duration(summary.duration_minutes),
                                summary.interaction_count
                            );
                            server::send_notification(&handle2, "Breather", &msg);
                        }
                    }
                }
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Breather")
        .run(|app, event| {
            match &event {
                RunEvent::ExitRequested { .. } => {
                    if let Some(mgr) = app.try_state::<Arc<StateManager>>() {
                        mgr.close_current_session();
                        mgr.save();
                    }
                }
                // Fires when macOS re-activates the app (e.g. user clicks a
                // notification banner or re-launches while running). For our
                // Accessory (menubar-only) app we use this as a cue to show
                // the main window.
                RunEvent::Reopen { .. } => {
                    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
                        window.show().ok();
                        window.set_focus().ok();
                    }
                }
                _ => {}
            }
        });
}

fn toggle_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        if window.is_visible().unwrap_or(false) {
            window.hide().ok();
        } else {
            window.show().ok();
            window.set_focus().ok();
        }
    } else {
        // Fallback in case the pre-created window got destroyed.
        create_window(app, true);
    }
}

fn create_window(app: &tauri::AppHandle, visible: bool) {
    if let Ok(window) = WebviewWindowBuilder::new(
        app,
        WINDOW_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .title("Breather")
    .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
    .resizable(false)
    .minimizable(false)
    .maximizable(false)
    .visible(visible)
    .build()
    {
        let win_handle = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                win_handle.hide().ok();
            }
        });
    }
}
