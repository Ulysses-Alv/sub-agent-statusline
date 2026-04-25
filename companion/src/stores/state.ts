import { createSignal } from "solid-js";
import type { StatuslineState } from "../lib/types";

// Types for PID discovery
export interface PidCandidate {
  pid: number;
  state_path: string;
  last_modified_ms: number | null;
  subagent_count: number;
}

// Re-export types from lib/types for convenience
export type { StatuslineState, ChildSessionState, ChildTokenState } from "../lib/types";

// Signals
export const [pids, setPids] = createSignal<PidCandidate[]>([]);
export const [selectedPid, setSelectedPid] = createSignal<number | null>(null);
export const [state, setState] = createSignal<StatuslineState | null>(null);
export const [watching, setWatching] = createSignal(false);
export const [error, setError] = createSignal<string | null>(null);
export const [expandedRows, setExpandedRows] = createSignal<Set<string>>(new Set());
export const [loading, setLoading] = createSignal(false);

// Helper to toggle row expansion
export function toggleRowExpansion(id: string): void {
  setExpandedRows((prev) => {
    const next = new Set(prev);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    return next;
  });
}

// Helper to check if row is expanded
export function isRowExpanded(id: string): boolean {
  return expandedRows().has(id);
}

// Clear state manually
export function clearState(): void {
  setState(null);
  setExpandedRows(new Set());
}

// Helper to get status counts from state
export function getStatusCounts(state: StatuslineState | null): { running: number; done: number; error: number } {
  if (!state) return { running: 0, done: 0, error: 0 };

  const children = Object.values(state.children);
  return {
    running: children.filter((c) => c.status === "running").length,
    done: children.filter((c) => c.status === "done").length,
    error: children.filter((c) => c.status === "error").length,
  };
}

// Delete a child from local state only (frontend signal)
// Does NOT write to disk - use deleteChild Tauri command for persistence
export function deleteChild(childId: string): void {
  setState((prev) => {
    if (!prev) return null;
    const newChildren = { ...prev.children };
    delete newChildren[childId];
    return { ...prev, children: newChildren };
  });
}