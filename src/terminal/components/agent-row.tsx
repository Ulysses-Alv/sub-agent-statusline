import React from "react";
import type { FC } from "react";
import { Box, Text } from "ink";
import type { ChildSessionState } from "../../state.js";

function formatDuration(elapsedMs: number | undefined): string {
  const totalSeconds = Math.max(0, Math.floor((elapsedMs ?? 0) / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  }

  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

interface AgentRowProps {
  agent: ChildSessionState;
  selected: boolean;
}

export const AgentRow: FC<AgentRowProps> = ({ agent, selected }) => {
  const statusIcon = agent.status === "done" ? "✓" : agent.status === "error" ? "✕" : "●";
  const statusColor = agent.color === "green" ? "green" : agent.color === "red" ? "red" : "yellow";
  const elapsed = formatDuration(agent.elapsedMs);

  return (
    <Box>
      <Text
        color={selected ? undefined : statusColor}
        dimColor={!selected && agent.status !== "running"}
        inverse={selected}
      >
        {statusIcon} {agent.title} {elapsed}
      </Text>
    </Box>
  );
};