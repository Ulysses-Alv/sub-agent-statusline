import { Component, Show } from "solid-js";

interface StatusBarProps {
  running: number;
  done: number;
  error: number;
  onClear?: () => void;
}

const StatusBar: Component<StatusBarProps> = (props) => {
  const hasCompleted = () => props.done > 0 || props.error > 0;

  return (
    <div class="status-bar">
      <div class="status-item">
        <span class="status-dot running" />
        <span>{props.running} running</span>
      </div>
      <div class="status-item">
        <span class="status-dot done" />
        <span>✓ {props.done} done</span>
      </div>
      <div class="status-item">
        <span class="status-dot error" />
        <span>✕ {props.error} error</span>
      </div>
      <Show when={hasCompleted() && props.onClear}>
        <button class="clear-btn" onClick={props.onClear} title="Clear completed">
          Clear
        </button>
      </Show>
    </div>
  );
};

export default StatusBar;