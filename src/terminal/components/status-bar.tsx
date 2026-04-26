import React from "react";
import type { FC } from "react";
import { Box, Text } from "ink";

interface StatusBarProps {
  running: number;
  done: number;
  error: number;
}

export const StatusBar: FC<StatusBarProps> = ({ running, done, error }) => {
  return (
    <Box>
      <Text color="yellow">● {running} running</Text>
      <Text color="gray"> · </Text>
      <Text color="green">✓ {done} done</Text>
      <Text color="gray"> · </Text>
      <Text color="red">✕ {error} error</Text>
    </Box>
  );
};