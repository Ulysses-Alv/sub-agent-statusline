import React from "react";
import type { FC } from "react";
import { Box, Text } from "ink";
import type { ChildSessionState } from "../../state.js";

interface ConfirmDialogProps {
  agent: ChildSessionState;
  onConfirm: () => void;
  onCancel: () => void;
}

export const ConfirmDialog: FC<ConfirmDialogProps> = ({ agent, onConfirm, onCancel }) => {
  return (
    <Box
      flexDirection="column"
      borderStyle="round"
      borderColor="cyan"
      padding={1}
      marginTop={1}
    >
      <Text>Delete "{agent.title}"? [y/N]</Text>
      <Text dimColor>Press y to confirm, n to cancel</Text>
    </Box>
  );
};