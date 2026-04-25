mod process;
mod state;
mod watcher;

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc as tokio_mpsc;

use process::{discover_pids_impl, PidCandidate};
use state::{delete_child as delete_child_impl, AppState, SharedAppState, StatuslineState};
use watcher::{read_state_now as read_state_now_impl, StateWatcher, WatchEvent};
use tauri_plugin_store::StoreExt;

/// Tauri command: Discover all running opencode-related processes.
/// Returns a list of PID candidates with their state file paths.
#[tauri::command]
async fn discover_pids() -> Result<Vec<PidCandidate>, String> {
    Ok(discover_pids_impl())
}

/// Tauri command: Start watching the state file for a specific PID.
/// Sets up file watching with debounce and emits state-updated events.
#[tauri::command]
async fn start_watching(
    pid: u32,
    app: AppHandle,
    shared_state: tauri::State<'_, SharedAppState>,
) -> Result<(), String> {
    // Create channel for watch events
    let (tx, mut rx) = tokio_mpsc::unbounded_channel::<WatchEvent>();

    // Clone app handle for event emission
    let app_clone = app.clone();

    // Spawn event handler
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                WatchEvent::StateUpdated(state) => {
                    let _ = app_clone.emit("state-updated", state);
                }
                WatchEvent::StateMissing => {
                    let _ = app_clone.emit("state-missing", pid);
                }
                WatchEvent::ParseError { error } => {
                    let _ = app_clone.emit("parse-error", serde_json::json!({
                        "pid": pid,
                        "error": error
                    }));
                }
            }
        }
    });

    // Create watcher and start watching
    let mut watcher = StateWatcher::new();

    // Clone tx for polling fallback
    let tx_for_polling = tx.clone();

    // Get app state for watcher
    let app_state_arc = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    match watcher.start_watching(pid, tx, app_state_arc.clone()).map_err(|e| e.to_string())? {
        Some(_) => {
            // notify watcher started successfully
        }
        None => {
            // notify failed, use polling fallback
            watcher.start_polling(pid, tx_for_polling, app_state_arc);
        }
    }

    // Store watcher in AppState so it doesn't get dropped
    {
        let mut state = shared_state.lock().map_err(|e| e.to_string())?;
        state.set_pid(Some(pid));
        state.watching = true;
        state.watcher = Some(watcher);
    }

    Ok(())
}

/// Tauri command: Stop watching the current state file.
/// Clears the current PID and stops the file watcher.
#[tauri::command]
async fn stop_watching(
    shared_state: tauri::State<'_, SharedAppState>,
) -> Result<(), String> {
    let mut state = shared_state.lock().map_err(|e| e.to_string())?;
    state.clear_state();
    Ok(())
}

/// Tauri command: Read the state file for a PID immediately.
/// Bypasses the debounce mechanism and returns the current state.
#[tauri::command]
async fn read_state_now(pid: u32) -> Result<Option<StatuslineState>, String> {
    Ok(read_state_now_impl(pid))
}

/// Tauri command: Delete a child subagent from the state file.
/// Removes the child entry from state.json for the given PID.
#[tauri::command]
async fn delete_child(pid: i32, child_id: String) -> Result<(), String> {
    delete_child_impl(pid as u32, &child_id)
}

/// Tauri command: Save the window position to persistent store.
/// Uses the app's managed store to persist position.
#[tauri::command]
async fn save_window_position(
    x: i32,
    y: i32,
    app: AppHandle,
) -> Result<(), String> {
    let store = app.store("window-position.json").map_err(|e| e.to_string())?;
    store.set("window_position", serde_json::json!({ "x": x, "y": y }));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Tauri command: Load the window position from persistent store.
/// Returns None if not set or if the position is off-screen.
#[tauri::command]
async fn load_window_position(
    app: AppHandle,
) -> Result<Option<(i32, i32)>, String> {
    let store = app.store("window-position.json").map_err(|e| e.to_string())?;

    match store.get("window_position") {
        Some(value) => {
            if let Some(obj) = value.as_object() {
                let x = obj.get("x").and_then(|v| v.as_i64()).map(|v| v as i32);
                let y = obj.get("y").and_then(|v| v.as_i64()).map(|v| v as i32);

                if let (Some(x), Some(y)) = (x, y) {
                    // Validate on-screen position
                    if is_position_on_screen(x, y) {
                        return Ok(Some((x, y)));
                    }
                }
            }
            Ok(None)
        }
        None => Ok(None),
    }
}

/// Check if a position is within primary monitor bounds.
fn is_position_on_screen(x: i32, y: i32) -> bool {
    // Reasonable bounds check - assume at least 800x600 minimum
    // and no more than 7680x4320 (8K max reasonable)
    if x < 0 || y < 0 || x > 7680 || y > 4320 {
        return false;
    }
    true
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shared_state: SharedAppState = Mutex::new(AppState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_process::init())
        .manage(shared_state)
        .invoke_handler(tauri::generate_handler![
            discover_pids,
            start_watching,
            stop_watching,
            read_state_now,
            delete_child,
            save_window_position,
            load_window_position,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let pids = discover_pids_impl();
                let _ = handle.emit("pids-changed", pids);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_pids_returns_vec() {
        // Just verify it returns something without panicking
        let result = discover_pids_impl();
        let _ = result.len();
    }

    #[test]
    fn test_is_position_on_screen_valid() {
        // Test positions within reasonable bounds
        assert!(is_position_on_screen(100, 100));
        assert!(is_position_on_screen(500, 400));
        assert!(is_position_on_screen(1920, 1080));
    }

    #[test]
    fn test_is_position_on_screen_negative() {
        // Negative positions should be considered invalid
        assert!(!is_position_on_screen(-100, 100));
        assert!(!is_position_on_screen(100, -100));
    }

    #[test]
    fn test_is_position_on_screen_too_large() {
        // Positions beyond reasonable bounds should be invalid
        assert!(!is_position_on_screen(8000, 100));
        assert!(!is_position_on_screen(100, 5000));
    }

    #[test]
    fn test_app_state_shared() {
        let state: SharedAppState = Mutex::new(AppState::new());
        let mut guard = state.lock().unwrap();
        guard.set_pid(Some(12345));
        assert_eq!(guard.current_pid, Some(12345));
    }

    #[test]
    fn test_pid_candidate_structure() {
        let candidate = PidCandidate {
            pid: 12345,
            state_path: "/tmp/opencode-subagent-statusline/pid-12345/state.json".to_string(),
            last_modified_ms: Some(1234567890),
            subagent_count: 3,
        };

        assert_eq!(candidate.pid, 12345);
        assert_eq!(candidate.subagent_count, 3);
        assert!(candidate.last_modified_ms.is_some());
    }
}
