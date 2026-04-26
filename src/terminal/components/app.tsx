import React, { useState } from "react";
import { Box, Text } from "ink";
import type { ChildSessionState, StatuslineState } from "../../state.js";
import { getCounts } from "../../state.js";
import { StatusBar } from "./status-bar.js";
import { AgentList } from "./agent-list.js";
import { ConfirmDialog } from "./confirm-dialog.js";

interface AppProps {
  state: StatuslineState;
  onDelete: (id: string) => void;
  onQuit: () => void;
}

export function App({ state, onDelete, onQuit }: AppProps): React.ReactElement {
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [confirmDialog, setConfirmDialog] = useState<ChildSessionState | null>(null);

  const agents = Object.values(state.children).sort((a, b) => {
    const rank = (c: ChildSessionState) => {
      if (c.status === "running") return 0;
      if (c.status === "error") return 1;
      return 2;
    };
    const diff = rank(a) - rank(b);
    if (diff !== 0) return diff;
    return b.updatedAt.localeCompare(a.updatedAt);
  });

  const counts = getCounts(state);

  const onConfirmDelete = (): void => {
    if (confirmDialog) {
      onDelete(confirmDialog.id);
      setConfirmDialog(null);
    }
  };

  const onCancelDelete = (): void => {
    setConfirmDialog(null);
  };

  return (
    <Box flexDirection="column">
      <Box flexDirection="row" justifyContent="space-between" paddingBottom={1}>
        <Text bold>Subagent Status</Text>
        <StatusBar
          running={counts.running}
          done={counts.done}
          error={counts.error}
        />
      </Box>
      <Box flexDirection="column" flexGrow={1}>
        <AgentList
          agents={agents}
          selectedIndex={selectedIndex}
        />
      </Box>
      {confirmDialog && (
        <ConfirmDialog
          agent={confirmDialog}
          onConfirm={onConfirmDelete}
          onCancel={onCancelDelete}
        />
      )}
      <Text dimColor>↑↓ navigate · d delete · q quit</Text>
    </Box>
  );
}