import React from "react";
import { render } from "ink";
import minimist from "minimist";
import { discoverLatestStatePath, watchState, loadState } from "./file-watcher.js";
import { App } from "./components/app.js";
import type { StatuslineState } from "../state.js";

interface CliFlags {
  openDesktop: boolean;
  terminal: boolean;
}

function parseFlags(): CliFlags {
  const args = minimist(process.argv.slice(2), {
    boolean: ["open-desktop", "terminal"],
    default: { terminal: true, "open-desktop": false },
  });

  return {
    openDesktop: args["open-desktop"] ?? false,
    terminal: args["terminal"] ?? true,
  };
}

async function main(): Promise<void> {
  const flags = parseFlags();

  if (flags.openDesktop) {
    console.log("Open desktop mode not yet implemented");
    process.exit(1);
  }

  let statePath: string | null;
  try {
    statePath = await discoverLatestStatePath();
    if (!statePath) {
      console.error("Could not discover state file");
      process.exit(2);
    }
  } catch (err) {
    console.error("Error discovering state:", err);
    process.exit(1);
  }

  const [state, setState] = React.useState<StatuslineState>({
    children: {},
    updatedAt: new Date().toISOString(),
  });

  const stopWatcher = await watchState(statePath, (newState) => {
    setState(newState);
  });

  const handleQuit = (): void => {
    stopWatcher();
    process.exit(0);
  };

  const handleDelete = (id: string): void => {
    console.log("Delete requested for:", id);
  };

  const { waitUntilExit } = render(
    React.createElement(App, {
      state: state,
      onDelete: handleDelete,
      onQuit: handleQuit,
    }),
  );

  await waitUntilExit();
}

main().catch((err) => {
  console.error("Fatal error:", err);
  process.exit(1);
});