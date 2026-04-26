import React from "react";
import type { FC } from "react";
import { Box, Text } from "ink";
import type { ChildSessionState } from "../../state.js";
import { AgentRow } from "./agent-row.js";

interface AgentListProps {
  agents: ChildSessionState[];
  selectedIndex: number;
}

interface AgentListProps {
  agents: ChildSessionState[];
  selectedIndex: number;
}

export const AgentList: FC<AgentListProps> = ({ agents, selectedIndex }) => {
  if (agents.length === 0) {
    return (
      <Box flexDirection="column" justifyContent="center" alignItems="center" flexGrow={1}>
        <Text dimColor>No agents yet</Text>
      </Box>
    );
  }

  return (
    <Box flexDirection="column">
      {agents.map((agent, index) => (
        <AgentRow
          key={agent.id}
          agent={agent}
          selected={index === selectedIndex}
        />
      ))}
    </Box>
  );
};