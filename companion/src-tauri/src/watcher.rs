//! File watching module using notify crate.
//! Implements debouncing and polling fallback for state.json changes.

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use tokio::sync::mpsc as tokio_mpsc;
use tokio::time::{sleep, Instant};

use crate::state::{AppState, StatuslineState};
use crate::process::resolve_state_path;

/// Duration to wait before emitting an event after the last change (debounce).
const DEBOUNCE_MS: u64 = 100;

/// Duration for polling fallback interval.
const POLL_INTERVAL_MS: u64 = 1000;

/// Event emitted when state file changes.
#[derive(Debug, Clone)]
pub enum WatchEvent {
    /// State file was modified and parsed successfully.
    StateUpdated(StatuslineState),
    /// State file was deleted or missing.
    StateMissing,
    /// Failed to parse the state file.
    ParseError { error: String },
}

/// File watcher with debounce support and polling fallback.
pub struct StateWatcher {
    /// The notify watcher handle.
    watcher: Option<RecommendedWatcher>,
    /// Channel to receive events from notify.
    receiver: Option<Receiver<Result<Event, notify::Error>>>,
    /// Currently watched PID.
    watched_pid: Option<u32>,
}

impl StateWatcher {
    /// Create a new StateWatcher.
    pub fn new() -> Self {
        Self {
            watcher: None,
            receiver: None,
            watched_pid: None,
        }
    }

    /// Start watching a state.json file for a given PID.
    /// Returns Ok(None) if notify initialization failed (will use polling fallback).
    pub fn start_watching(
        &mut self,
        pid: u32,
        state_tx: tokio_mpsc::UnboundedSender<WatchEvent>,
        app_state: std::sync::Arc<tokio::sync::Mutex<AppState>>,
    ) -> Result<Option<()>, String> {
        // Stop any existing watcher
        self.stop_watching();

        let state_path = resolve_state_path(pid);
        
        // Check if file exists first
        if !state_path.exists() {
            return Err(format!("State file does not exist: {}", state_path.display()));
        }

        self.watched_pid = Some(pid);

        // Try to set up notify watcher
        let (event_tx, event_rx) = channel();
        self.receiver = Some(event_rx);

        let state_path_clone = state_path.clone();
        let state_tx_clone = state_tx.clone();
        let app_state_clone = app_state.clone();

        let result: Result<notify::RecommendedWatcher, notify::Error> = Watcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = event_tx.send(Ok(event));
                }
            },
            Config::default(),
        );

        match result {
            Ok(mut watcher) => {
                // Watch the directory containing the state file
                if let Some(parent) = state_path.parent() {
                    watcher
                        .watch(parent, RecursiveMode::NonRecursive)
                        .map_err(|e| format!("Failed to watch directory: {}", e))?;
                }

                self.watcher = Some(watcher);

                // Start the debounce processing loop
                let rx = self.receiver.take().unwrap();
                tokio::spawn(async move {
                    StateWatcher::process_notify_events(
                        rx,
                        state_path_clone,
                        state_tx_clone,
                        app_state_clone,
                    ).await;
                });

                Ok(Some(()))
            }
            Err(_e) => {
                // notify failed, fall back to polling
                Ok(None)
            }
        }
    }

    /// Process events from notify with debouncing.
    async fn process_notify_events(
        receiver: Receiver<Result<Event, notify::Error>>,
        state_path: PathBuf,
        state_tx: tokio_mpsc::UnboundedSender<WatchEvent>,
        app_state: std::sync::Arc<tokio::sync::Mutex<AppState>>,
    ) {
        let mut debounce_deadline: Option<Instant> = None;
        let mut pending_event = false;

        loop {
            let timeout = debounce_deadline
                .map(|d| {
                    let remaining = d.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        Some(Duration::from_millis(DEBOUNCE_MS))
                    } else {
                        Some(remaining)
                    }
                })
                .flatten();

            // Use recv_timeout to wait for events or debounce expiry
            let msg = match timeout {
                Some(t) => {
                    // Non-blocking check first
                    match receiver.try_recv() {
                        Ok(msg) => Some(msg),
                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                            // Wait with timeout using sleep
                            sleep(t).await;
                            // After sleeping, check again
                            match receiver.recv() {
                                Ok(msg) => Some(msg),
                                Err(_) => None, // Channel closed
                            }
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => None,
                    }
                }
                None => {
                    // No pending debounce, wait indefinitely
                    match receiver.recv() {
                        Ok(msg) => Some(msg),
                        Err(_) => None,
                    }
                }
            };

            // Process received message
            if let Some(msg) = msg {
                match msg {
                    Ok(event) => {
                        // Check if this event is about our state file
                        let relevant = event.paths.iter().any(|p| {
                            p.file_name().map(|n| n == "state.json").unwrap_or(false)
                        });
                        
                        if relevant {
                            // Set debounce timer
                            debounce_deadline = Some(Instant::now() + Duration::from_millis(DEBOUNCE_MS));
                            pending_event = true;
                        }
                    }
                    Err(e) => {
                        let _ = state_tx.send(WatchEvent::ParseError { 
                            error: format!("Notify error: {}", e) 
                        });
                    }
                }
            } else if pending_event && debounce_deadline.map(|d| Instant::now() >= d).unwrap_or(false) {
                // Debounce timer expired, emit the event
                debounce_deadline = None;
                pending_event = false;
                
                // Read and parse the current state
                match fs::read_to_string(&state_path) {
                    Ok(contents) => {
                        match serde_json::from_str::<StatuslineState>(&contents) {
                            Ok(state) => {
                                let _ = state_tx.send(WatchEvent::StateUpdated(state.clone()));
                                
                                // Update app state
                                let mut app = app_state.lock().await;
                                app.set_state(Some(state));
                            }
                            Err(e) => {
                                let _ = state_tx.send(WatchEvent::ParseError {
                                    error: format!("JSON parse error: {}", e)
                                });
                            }
                        }
                    }
                    Err(_) => {
                        // File might have been deleted
                        let _ = state_tx.send(WatchEvent::StateMissing);
                    }
                }
            }
        }
    }

    /// Start polling fallback when notify fails.
    pub fn start_polling(
        &mut self,
        pid: u32,
        state_tx: tokio_mpsc::UnboundedSender<WatchEvent>,
        app_state: std::sync::Arc<tokio::sync::Mutex<AppState>>,
    ) {
        self.watched_pid = Some(pid);

        tokio::spawn(async move {
            let mut last_modified: Option<u64> = None;
            
            loop {
                sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
                
                let state_path_current = resolve_state_path(pid);
                
                if !state_path_current.exists() {
                    let _ = state_tx.send(WatchEvent::StateMissing);
                    last_modified = None;
                    continue;
                }
                
                // Get current modified time
                let current_modified = fs::metadata(&state_path_current)
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                        duration.as_millis() as u64
                    });
                
                // Only emit if changed since last check
                if current_modified != last_modified {
                    last_modified = current_modified;
                    
                    match fs::read_to_string(&state_path_current) {
                        Ok(contents) => {
                            match serde_json::from_str::<StatuslineState>(&contents) {
                                Ok(state) => {
                                    let _ = state_tx.send(WatchEvent::StateUpdated(state.clone()));
                                    
                                    let mut app = app_state.lock().await;
                                    app.set_state(Some(state));
                                }
                                Err(e) => {
                                    let _ = state_tx.send(WatchEvent::ParseError {
                                        error: format!("JSON parse error: {}", e)
                                    });
                                }
                            }
                        }
                        Err(_) => {
                            let _ = state_tx.send(WatchEvent::StateMissing);
                        }
                    }
                }
            }
        });
    }

    /// Stop watching and clean up resources.
    pub fn stop_watching(&mut self) {
        self.watcher = None;
        self.receiver = None;
        self.watched_pid = None;
    }

    /// Get the currently watched PID.
    #[allow(dead_code)]
    pub fn watched_pid(&self) -> Option<u32> {
        self.watched_pid
    }
}

impl Default for StateWatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Read state.json for a given PID immediately (bypass debounce).
pub fn read_state_now(pid: u32) -> Option<StatuslineState> {
    let state_path = resolve_state_path(pid);
    
    if !state_path.exists() {
        return None;
    }
    
    let contents = fs::read_to_string(&state_path).ok()?;
    serde_json::from_str(&contents).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_watcher_new() {
        let watcher = StateWatcher::new();
        assert!(watcher.watched_pid.is_none());
        assert!(watcher.watcher.is_none());
    }

    #[test]
    fn test_resolve_state_path() {
        let path = resolve_state_path(12345);
        let expected = std::env::temp_dir()
            .join("opencode-subagent-statusline")
            .join("pid-12345")
            .join("state.json");
        assert_eq!(path, expected);
    }

    #[test]
    fn test_read_state_now_nonexistent() {
        let result = read_state_now(99999);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_error_format() {
        let error = WatchEvent::ParseError {
            error: "JSON parse error: expected }".to_string()
        };

        match error {
            WatchEvent::ParseError { error: msg } => {
                assert!(msg.contains("JSON parse error"));
            }
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_watch_event_variants() {
        // Test StateUpdated variant
        let state = StatuslineState {
            children: std::collections::HashMap::new(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let event = WatchEvent::StateUpdated(state);
        assert!(matches!(event, WatchEvent::StateUpdated(_)));

        // Test StateMissing variant
        let event = WatchEvent::StateMissing;
        assert!(matches!(event, WatchEvent::StateMissing));

        // Test ParseError variant
        let event = WatchEvent::ParseError { error: "test".to_string() };
        assert!(matches!(event, WatchEvent::ParseError { .. }));
    }

    #[test]
    fn test_debounce_timing() {
        // Verify the constants are sensible
        assert_eq!(DEBOUNCE_MS, 100);
        assert_eq!(POLL_INTERVAL_MS, 1000);
    }
}