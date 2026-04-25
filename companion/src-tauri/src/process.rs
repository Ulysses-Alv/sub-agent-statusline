//! PID discovery module using sysinfo crate.
//! Scans all processes for opencode-related processes and resolves their state paths.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use sysinfo::System;

/// A candidate process that may be running an opencode subagent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidCandidate {
    /// Process ID
    pub pid: u32,
    /// Path to the state.json file for this PID
    pub state_path: String,
    /// Last modified time of state.json in milliseconds (None if doesn't exist)
    pub last_modified_ms: Option<u64>,
    /// Number of subagents currently tracked in state.json
    pub subagent_count: usize,
}

/// Find the latest state.json file in the temp directory.
/// Instead of searching by PID, finds the most recently modified directory.
pub fn resolve_latest_state_path() -> Option<PathBuf> {
    let mut base = std::env::temp_dir();
    base.push("opencode-subagent-statusline");

    let entries = fs::read_dir(&base).ok()?;

    let mut latest: Option<(SystemTime, PathBuf)> = None;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    match &latest {
                        Some((latest_time, _)) if *latest_time >= modified => {}
                        _ => {
                            latest = Some((modified, path));
                        }
                    }
                }
            }
        }
    }

    let latest_dir = latest?.1;

    let mut state_path = latest_dir;
    state_path.push("state.json");

    Some(state_path)
}

/// Resolve the state file path for a given PID.
/// Format: `${TMPDIR}/opencode-subagent-statusline/pid-${PID}/state.json`
/// Note: For discovery, use resolve_latest_state_path() instead.
pub fn resolve_state_path(pid: u32) -> PathBuf {
    let base = std::env::temp_dir();
    base.join("opencode-subagent-statusline")
        .join(format!("pid-{}", pid))
        .join("state.json")
}

/// Get last modified time of a file in milliseconds.
fn get_last_modified_ms(path: &PathBuf) -> Option<u64> {
    fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|t| {
            let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
            duration.as_millis() as u64
        })
}

/// Count subagents in a state.json file by reading the children map.
fn count_subagents(state_path: &PathBuf) -> usize {
    if let Ok(contents) = fs::read_to_string(state_path) {
        if let Ok(state) = serde_json::from_str::<serde_json::Value>(&contents) {
            if let Some(children) = state.get("children").and_then(|c| c.as_object()) {
                return children.len();
            }
        }
    }
    0
}

/// Discover state.json by finding the latest directory.
/// Returns a single PidCandidate for the most recently modified state.json.
pub fn discover_pids_impl() -> Vec<PidCandidate> {
    println!("[DEBUG discover_pids] Searching for latest state.json...");

    let state_path = match resolve_latest_state_path() {
        Some(path) => {
            println!("[DEBUG discover_pids] Found latest state.json at: {}", path.display());
            path
        }
        None => {
            println!("[DEBUG discover_pids] No state.json found in temp directory");
            return Vec::new();
        }
    };

    // Extract PID from directory name (e.g., "pid-12988" -> 12988)
    if let Some(parent) = state_path.parent() {
        let dir_name = parent.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if dir_name.starts_with("pid-") {
            if let Ok(pid) = dir_name[4..].parse::<u32>() {
                let last_modified_ms = get_last_modified_ms(&state_path);
                let subagent_count = count_subagents(&state_path);

                println!("[DEBUG discover_pids] Extracted PID {} with {} subagents", pid, subagent_count);

                return vec![PidCandidate {
                    pid,
                    state_path: state_path.to_string_lossy().to_string(),
                    last_modified_ms,
                    subagent_count,
                }];
            }
        }
    }

    println!("[DEBUG discover_pids] Could not extract PID from path");
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_is_opencode_main_process_exact_match() {
        assert!(is_opencode_main_process("opencode"));
        assert!(is_opencode_main_process("OpenCode"));
        assert!(is_opencode_main_process("OPENCODE"));
        assert!(is_opencode_main_process("opencode.exe"));
        assert!(is_opencode_main_process("OpenCode.exe"));
        assert!(is_opencode_main_process("opencode-cli"));
        assert!(is_opencode_main_process("opencode-cli.exe"));
        assert!(is_opencode_main_process("opencode_cli.exe"));
    }

    #[test]
    fn test_is_opencode_main_process_with_suffix() {
        // These should NOT match - they're variants/helper processes
        assert!(!is_opencode_main_process("opencode-helper"));
        assert!(!is_opencode_main_process("opencode_helper"));
        assert!(!is_opencode_main_process("opencode-service"));
        assert!(!is_opencode_main_process("opencode_tool"));
        assert!(!is_opencode_main_process("my-opencode"));
        assert!(!is_opencode_main_process("my_opencode"));
    }

    #[test]
    fn test_is_opencode_main_process_similar_names() {
        // Should not match things that just contain opencode as substring
        assert!(!is_opencode_main_process("not-opencode"));
        assert!(!is_opencode_main_process("fake-opencode-run"));
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
    fn test_count_subagents_empty_children() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");
        
        let json = r#"{"children": {}, "updatedAt": "2024-01-01T00:00:00Z"}"#;
        fs::write(&state_path, json).unwrap();
        
        assert_eq!(count_subagents(&state_path), 0);
    }

    #[test]
    fn test_count_subagents_with_children() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");
        
        let json = r#"{
            "children": {
                "agent1": {"id": "agent1"},
                "agent2": {"id": "agent2"},
                "agent3": {"id": "agent3"}
            },
            "updatedAt": "2024-01-01T00:00:00Z"
        }"#;
        fs::write(&state_path, json).unwrap();
        
        assert_eq!(count_subagents(&state_path), 3);
    }

    #[test]
    fn test_count_subagents_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");
        
        fs::write(&state_path, "not valid json").unwrap();
        
        assert_eq!(count_subagents(&state_path), 0);
    }

    #[test]
    fn test_count_subagents_missing_children_field() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");
        
        let json = r#"{"updatedAt": "2024-01-01T00:00:00Z"}"#;
        fs::write(&state_path, json).unwrap();
        
        assert_eq!(count_subagents(&state_path), 0);
    }

    #[test]
    fn test_get_last_modified_ms() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        fs::write(&file_path, "content").unwrap();
        
        let ms = get_last_modified_ms(&file_path);
        assert!(ms.is_some());
        assert!(ms.unwrap() > 0);
    }

    #[test]
    fn test_get_last_modified_ms_nonexistent() {
        let path = PathBuf::from("nonexistent_file_12345.txt");
        assert_eq!(get_last_modified_ms(&path), None);
    }

    #[test]
    fn test_discover_pids_returns_without_panic() {
        let results = discover_pids_impl();
        // Just verify it returns without panicking
        // Actual matching depends on what's running on the system
        let _ = results.len();
    }
}