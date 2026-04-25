import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import type { PidCandidate, StatuslineState } from "../stores/state";

// Commands
export async function discoverPids(): Promise<PidCandidate[]> {
  return invoke("discover_pids");
}

export async function startWatching(pid: number): Promise<void> {
  return invoke("start_watching", { pid });
}

export async function stopWatching(): Promise<void> {
  return invoke("stop_watching");
}

export async function readStateNow(pid: number): Promise<StatuslineState | null> {
  return invoke("read_state_now", { pid });
}

export async function deleteChild(pid: number, childId: string): Promise<void> {
  return invoke("delete_child", { pid, childId });
}

export async function saveWindowPosition(x: number, y: number): Promise<void> {
  return invoke("save_window_position", { x, y });
}

export async function loadWindowPosition(): Promise<[number, number] | null> {
  return invoke("load_window_position");
}

// Events
export async function onStateUpdated(handler: (state: StatuslineState) => void): Promise<UnlistenFn> {
  return listen<StatuslineState>("state-updated", (event) => handler(event.payload));
}

export async function onPidsChanged(handler: (pids: PidCandidate[]) => void): Promise<UnlistenFn> {
  return listen<PidCandidate[]>("pids-changed", (event) => handler(event.payload));
}

export async function onStateMissing(handler: (pid: number) => void): Promise<UnlistenFn> {
  return listen<number>("state-missing", (event) => handler(event.payload));
}

export async function onParseError(handler: (error: { pid: number; error: string }) => void): Promise<UnlistenFn> {
  return listen<{ pid: number; error: string }>("parse-error", (event) => handler(event.payload));
}