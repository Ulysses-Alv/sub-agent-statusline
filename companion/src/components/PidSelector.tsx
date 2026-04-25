import { Component } from "solid-js";
import { pids, selectedPid } from "../stores/state";

interface PidSelectorProps {
  onSelect: (pid: number) => void;
}

const PidSelector: Component<PidSelectorProps> = (props) => {
  const sortedPids = () => {
    const arr = pids();
    // Sort by last_modified_ms descending (most recent first)
    return [...arr].sort((a, b) => {
      const aTime = a.last_modified_ms ?? 0;
      const bTime = b.last_modified_ms ?? 0;
      return bTime - aTime;
    });
  };

  const handleChange = (e: Event) => {
    const target = e.target as HTMLSelectElement;
    const pid = parseInt(target.value, 10);
    props.onSelect(pid);
  };

  return (
    <div class="pid-selector">
      <select value={selectedPid() ?? ""} onChange={handleChange}>
        <option value="" disabled>Select a process instance...</option>
        {sortedPids().map((candidate) => (
          <option value={candidate.pid.toString()}>
            PID {candidate.pid} — {candidate.subagent_count} subagents
          </option>
        ))}
      </select>
    </div>
  );
};

export default PidSelector;