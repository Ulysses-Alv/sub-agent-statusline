//! Shared state management for the companion app.
//! Thread-safe state with Mutex wrapper.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;

use crate::watcher::StateWatcher;

/// Child token statistics for a subagent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildTokenState {
    #[serde(rename = "input")]
    pub input: Option<u32>,
    #[serde(rename = "output")]
    pub output: Option<u32>,
    #[serde(rename = "total")]
    pub total: Option<u32>,
    #[serde(rename = "contextPercent")]
    pub context_percent: Option<f32>,
}

/// Status of a child subagent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChildStatus {
    Running,
    Done,
    Error,
}

/// A single child subagent session state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChildSessionState {
    pub id: String,
    pub title: String,
    #[serde(rename = "parentID")]
    pub parent_id: String,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    pub source: Option<String>, // "session" | "subtask" | "tool"
    pub status: ChildStatus,
    pub color: String, // "yellow" | "green" | "red"
    pub started_at: String,
    pub updated_at: String,
    pub ended_at: Option<String>,
    pub elapsed_ms: Option<u64>,
    pub tokens: Option<ChildTokenState>,
}

/// The top-level statusline state structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatuslineState {
    pub children: HashMap<String, ChildSessionState>,
    pub updated_at: String,
}

/// Application shared state.
/// This struct is wrapped in a Mutex for thread-safe access.
pub struct AppState {
    /// Currently watched PID (if any)
    pub current_pid: Option<u32>,
    /// The most recently parsed state
    pub parsed_state: Option<StatuslineState>,
    /// Last valid state that was successfully parsed
    pub last_valid_state: Option<StatuslineState>,
    /// Whether we're currently watching a file
    pub watching: bool,
    /// The file watcher (kept alive to prevent dropping)
    pub watcher: Option<StateWatcher>,
}

impl AppState {
    /// Create a new AppState with default values.
    pub fn new() -> Self {
        Self {
            current_pid: None,
            parsed_state: None,
            last_valid_state: None,
            watching: false,
            watcher: None,
        }
    }

    /// Set the current PID being watched.
    pub fn set_pid(&mut self, pid: Option<u32>) {
        self.current_pid = pid;
    }

    /// Get a reference to the current state.
    #[allow(dead_code)]
    pub fn get_state(&self) -> Option<&StatuslineState> {
        self.parsed_state.as_ref()
    }

    /// Set the current state and update last_valid_state if valid.
    pub fn set_state(&mut self, state: Option<StatuslineState>) {
        self.parsed_state = state.clone();
        if self.parsed_state.is_some() {
            self.last_valid_state = self.parsed_state.clone();
        }
    }

    /// Clear all state but preserve last_valid_state.
    pub fn clear_state(&mut self) {
        self.parsed_state = None;
        self.current_pid = None;
        self.watching = false;
        self.watcher = None;
    }

    /// Get the last valid state.
    #[allow(dead_code)]
    pub fn get_last_valid_state(&self) -> Option<&StatuslineState> {
        self.last_valid_state.as_ref()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper type for thread-safe AppState access.
pub type SharedAppState = Mutex<AppState>;

/// Delete a child from the state.json file for a given PID.
pub fn delete_child(pid: u32, child_id: &str) -> Result<(), String> {
    let state_path = crate::process::resolve_state_path(pid);

    if !state_path.exists() {
        return Err(format!("State file does not exist for PID {}", pid));
    }

    let contents = fs::read_to_string(&state_path)
        .map_err(|e| format!("Failed to read state file: {}", e))?;

    let mut state: StatuslineState = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse state JSON: {}", e))?;

    // Remove the child
    let existed = state.children.remove(child_id).is_some();

    if !existed {
        return Err(format!("Child {} not found in state", child_id));
    }

    // Write back
    let json = serde_json::to_string_pretty(&state)
        .map_err(|e| format!("Failed to serialize state: {}", e))?;

    fs::write(&state_path, json)
        .map_err(|e| format!("Failed to write state file: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert!(state.current_pid.is_none());
        assert!(state.parsed_state.is_none());
        assert!(state.last_valid_state.is_none());
        assert!(!state.watching);
    }

    #[test]
    fn test_set_pid() {
        let mut state = AppState::new();
        state.set_pid(Some(12345));
        assert_eq!(state.current_pid, Some(12345));
        
        state.set_pid(None);
        assert!(state.current_pid.is_none());
    }

    #[test]
    fn test_set_state() {
        let mut state = AppState::new();
        
        let test_state = StatuslineState {
            children: HashMap::new(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        
        state.set_state(Some(test_state.clone()));
        assert!(state.parsed_state.is_some());
        assert!(state.last_valid_state.is_some());
        
        state.set_state(None);
        assert!(state.parsed_state.is_none());
        // last_valid_state should be preserved
        assert!(state.last_valid_state.is_some());
    }

    #[test]
    fn test_clear_state() {
        let mut state = AppState::new();
        state.set_pid(Some(12345));
        state.watching = true;
        
        state.clear_state();
        
        assert!(state.current_pid.is_none());
        assert!(state.parsed_state.is_none());
        assert!(!state.watching);
        // last_valid_state preserved
        assert!(state.last_valid_state.is_none());
    }

    #[test]
    fn test_statusline_state_parsing() {
        let json = r#"{
            "children": {
                "agent1": {
                    "id": "agent1",
                    "title": "Test Agent",
                    "parentId": "parent1",
                    "status": "running",
                    "color": "yellow",
                    "startedAt": "2024-01-01T00:00:00Z",
                    "updatedAt": "2024-01-01T00:00:01Z"
                }
            },
            "updatedAt": "2024-01-01T00:00:01Z"
        }"#;

        let state: StatuslineState = serde_json::from_str(json).unwrap();
        assert_eq!(state.children.len(), 1);
        assert_eq!(state.updated_at, "2024-01-01T00:00:01Z");
    }

    #[test]
    fn test_child_status_deserialization() {
        // ChildStatus is a plain enum, serialize directly to/from string value
        let json = r#""running""#;
        let status: ChildStatus = serde_json::from_str(json).unwrap();
        assert!(matches!(status, ChildStatus::Running));

        let json = r#""done""#;
        let status: ChildStatus = serde_json::from_str(json).unwrap();
        assert!(matches!(status, ChildStatus::Done));

        let json = r#""error""#;
        let status: ChildStatus = serde_json::from_str(json).unwrap();
        assert!(matches!(status, ChildStatus::Error));
    }

    #[test]
    fn test_child_token_state_parsing() {
        let json = r#"{
            "input": 100,
            "output": 200,
            "total": 300,
            "contextPercent": 75.5
        }"#;
        
        let tokens: ChildTokenState = serde_json::from_str(json).unwrap();
        assert_eq!(tokens.input, Some(100));
        assert_eq!(tokens.output, Some(200));
        assert_eq!(tokens.total, Some(300));
        assert_eq!(tokens.context_percent, Some(75.5));
    }

    #[test]
    fn test_child_session_state_with_all_fields() {
        let json = r#"{
            "id": "agent1",
            "title": "Test Agent",
            "parentId": "parent1",
            "messageID": "msg123",
            "source": "session",
            "status": "running",
            "color": "green",
            "startedAt": "2024-01-01T00:00:00Z",
            "updatedAt": "2024-01-01T00:00:01Z",
            "endedAt": null,
            "elapsedMs": 5000,
            "tokens": {
                "input": 100,
                "output": 200,
                "total": 300,
                "contextPercent": 50.0
            }
        }"#;
        
        let state: ChildSessionState = serde_json::from_str(json).unwrap();
        assert_eq!(state.id, "agent1");
        assert_eq!(state.message_id, Some("msg123".to_string()));
        assert_eq!(state.source, Some("session".to_string()));
        assert!(matches!(state.status, ChildStatus::Running));
        assert_eq!(state.color, "green");
        assert_eq!(state.elapsed_ms, Some(5000));
        assert!(state.tokens.is_some());
    }
}