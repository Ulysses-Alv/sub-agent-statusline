import { Component, Show, createMemo } from "solid-js";
import type { ChildSessionState } from "../stores/state";
import { toggleRowExpansion, isRowExpanded, deleteChild as deleteChildSignal } from "../stores/state";
import { deleteChild as deleteChildCmd } from "../lib/tauri";
import { selectedPid } from "../stores/state";

interface SubagentRowProps {
  child: ChildSessionState;
}

function formatElapsed(ms: number | undefined): string {
  if (ms === undefined) return "—";
  const seconds = Math.floor(ms / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);

  if (hours > 0) {
    return `${hours}h ${minutes % 60}m`;
  } else if (minutes > 0) {
    return `${minutes}m ${seconds % 60}s`;
  } else {
    return `${seconds}s`;
  }
}

function formatTokens(tokens: ChildSessionState["tokens"]): string {
  if (!tokens) return "";
  const parts: string[] = [];
  if (tokens.input !== undefined) parts.push(`in: ${tokens.input}`);
  if (tokens.output !== undefined) parts.push(`out: ${tokens.output}`);
  if (tokens.total !== undefined) parts.push(`total: ${tokens.total}`);
  if (tokens.contextPercent !== undefined) parts.push(`ctx: ${tokens.contextPercent}%`);
  return parts.join(" · ");
}

const SubagentRow: Component<SubagentRowProps> = (props) => {
  const expanded = createMemo(() => isRowExpanded(props.child.id));

  const statusDotClass = () => {
    switch (props.child.status) {
      case "running": return "status-dot running";
      case "done": return "status-dot done";
      case "error": return "status-dot error";
    }
  };

  const canDelete = () => props.child.status === "done" || props.child.status === "error";

  const handleDelete = async (e: MouseEvent) => {
    e.stopPropagation();
    const pid = selectedPid();
    if (!pid) return;
    deleteChildSignal(props.child.id);
    try {
      await deleteChildCmd(pid, props.child.id);
    } catch {
      // Silently ignore - local state already updated
    }
  };

  const handleForceClear = async (e: MouseEvent) => {
    e.stopPropagation();
    const confirmed = confirm("Force delete this agent from state.json? This cannot be undone.");
    if (!confirmed) return;
    const pid = selectedPid();
    if (!pid) return;
    deleteChildSignal(props.child.id);
    try {
      await deleteChildCmd(pid, props.child.id);
    } catch {
      // Silently ignore - local state already updated
    }
  };

  return (
    <div
      class={`subagent-row ${expanded() ? "expanded" : ""}`}
      onClick={() => toggleRowExpansion(props.child.id)}
    >
      <div class="subagent-header">
        <span class={statusDotClass()} />
        <span class="subagent-title">{props.child.title}</span>
        <span>{formatElapsed(props.child.elapsedMs)}</span>
        <Show when={canDelete()}>
          <button class="delete-btn" onClick={handleDelete} title="Remove from list">×</button>
        </Show>
        <button class="force-clear-btn" onClick={handleForceClear} title="Force delete from state">⚡</button>
      </div>
      <Show when={expanded()}>
        <div class="subagent-meta">
          <div>Status: {props.child.status}</div>
          <Show when={props.child.source}>
            <div>Source: {props.child.source}</div>
          </Show>
          <Show when={props.child.messageID}>
            <div>Message ID: {props.child.messageID}</div>
          </Show>
          <Show when={props.child.tokens}>
            <div>Tokens: {formatTokens(props.child.tokens)}</div>
          </Show>
          <div>Started: {new Date(props.child.startedAt).toLocaleTimeString()}</div>
          <Show when={props.child.endedAt}>
            <div>Ended: {new Date(props.child.endedAt!).toLocaleTimeString()}</div>
          </Show>
        </div>
      </Show>
    </div>
  );
};

export default SubagentRow;