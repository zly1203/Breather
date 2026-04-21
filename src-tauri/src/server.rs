use axum::{extract::State as AxumState, http::StatusCode, routing::{get, post}, Json, Router};
use std::sync::Arc;
use crate::state::StateManager;
use crate::rules::{character_state, check_fatigue, mark_reminded};
use crate::messages::get_reminder;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
#[cfg(not(target_os = "macos"))]
use tauri_plugin_notification::NotificationExt;

pub struct ServerState {
    pub state_mgr: Arc<StateManager>,
    pub app_handle: AppHandle,
}

/// Start the HTTP server on localhost:17422 for receiving hook events.
pub async fn start_server(state_mgr: Arc<StateManager>, app_handle: AppHandle) {
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:17422").await {
        Ok(l) => l,
        Err(e) => {
            log::error!("Cannot bind port 17422: {}", e);
            send_notification(
                &app_handle,
                "Breather",
                "Port 17422 is in use. Another Breather instance is already running.",
            );
            // Give the notification a moment to dispatch, then exit.
            // We can't keep running: without the port, session tracking is dead.
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            app_handle.exit(1);
            return;
        }
    };

    log::info!("Breather server listening on 127.0.0.1:17422");

    let server_state = Arc::new(ServerState { state_mgr, app_handle });
    let app = Router::new()
        .route("/event", post(handle_event))
        .route("/status", get(handle_status))
        .with_state(server_state);

    axum::serve(listener, app).await.ok();
}

/// POST /event: called by Claude Code Stop hook on each conversation turn.
async fn handle_event(
    AxumState(server): AxumState<Arc<ServerState>>,
    Json(_body): Json<Value>,
) -> StatusCode {
    let (is_new, _prev_summary) = server.state_mgr.record_interaction();

    // Notify frontend to update UI (session card, today total, history).
    server.app_handle.emit("session-updated", ()).ok();

    // If this interaction just started a new session, skip fatigue check:
    // a brand-new session has no fatigue to detect yet, and we deliberately
    // don't send any "welcome back" notification (the UI shows everything).
    if is_new {
        server.state_mgr.save();
        return StatusCode::OK;
    }

    // Check fatigue score and intervene if needed.
    if let Some(trigger) = check_fatigue(&server.state_mgr) {
        let message = get_reminder(&trigger.severity, &trigger.reason);
        send_notification(&server.app_handle, "Breather", message);
        mark_reminded(&server.state_mgr);
    }

    server.state_mgr.save();
    StatusCode::OK
}

/// GET /status: returns current session stats as JSON.
async fn handle_status(
    AxumState(server): AxumState<Arc<ServerState>>,
) -> Json<Value> {
    let character = character_state(&server.state_mgr);
    match server.state_mgr.get_stats() {
        Some(stats) => {
            let mut v = json!(stats);
            v["character"] = json!(character);
            Json(v)
        }
        None => Json(json!({"active": false, "character": character})),
    }
}

/// Send a notification. On macOS we route through `mac-notification-sys`
/// directly so we can react to clicks (the Tauri plugin discards the
/// user's response). Clicking the notification shows the main window.
pub fn send_notification(app: &AppHandle, title: &str, body: &str) {
    let app = app.clone();
    let title = title.to_string();
    let body = body.to_string();

    #[cfg(target_os = "macos")]
    std::thread::spawn(move || {
        let result = mac_notification_sys::Notification::default()
            .title(&title)
            .message(&body)
            .send();
        match result {
            Ok(mac_notification_sys::NotificationResponse::Click) => {
                log::info!("notification clicked: {}", title);
                let for_main = app.clone();
                let _ = app.run_on_main_thread(move || {
                    if let Some(w) = for_main.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                });
            }
            Ok(_) => log::info!("notification dismissed: {}", title),
            Err(e) => log::error!("notification failed: {}: {}", title, e),
        }
    });

    #[cfg(not(target_os = "macos"))]
    {
        match app.notification().builder().title(&title).body(&body).show() {
            Ok(_) => log::info!("notification dispatched: {}", title),
            Err(e) => log::error!("notification failed: {}: {}", title, e),
        }
    }
}

/// Bind the notification system to our app bundle identifier once at startup.
/// This ensures notifications show the Breather icon and tie back to this app
/// rather than appearing as an anonymous "Script Editor" notification.
#[cfg(target_os = "macos")]
pub fn init_macos_notifications(bundle_id: &str) {
    if let Err(e) = mac_notification_sys::set_application(bundle_id) {
        log::warn!("set_application failed: {}", e);
    }
}

/// Format duration in minutes as a human-readable string.
pub fn format_duration(minutes: u32) -> String {
    if minutes == 0 {
        return "<1m".to_string();
    }
    if minutes < 60 {
        return format!("{}m", minutes);
    }
    let h = minutes / 60;
    let m = minutes % 60;
    if m == 0 {
        format!("{}h", h)
    } else {
        format!("{}h {}m", h, m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_cases() {
        assert_eq!(format_duration(0), "<1m");
        assert_eq!(format_duration(1), "1m");
        assert_eq!(format_duration(59), "59m");
        assert_eq!(format_duration(60), "1h");
        assert_eq!(format_duration(61), "1h 1m");
        assert_eq!(format_duration(90), "1h 30m");
        assert_eq!(format_duration(120), "2h");
        assert_eq!(format_duration(125), "2h 5m");
        assert_eq!(format_duration(599), "9h 59m");
    }
}
