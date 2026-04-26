import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import os from "node:os";
import { readdirSync, statSync } from "node:fs";
import type { StatuslineState, ChildSessionState } from "../state.js";
import { createEmptyState } from "../state.js";

const STATUS_DIRNAME = "opencode-subagent-statusline";

export async function discoverLatestStatePath(): Promise<string | null> {
  const tmpDir = os.tmpdir();
  let latestPath: string | null = null;
  let latestMtime = 0;

  try {
    const entries = readdirSync(tmpDir, { withFileTypes: true });
    const pidFolders = entries.filter(
      (entry) =>
        entry.isDirectory() &&
        (entry.name.startsWith("pid-") || entry.name.startsWith("opencode-subagent-statusline")),
    );

    for (const folder of pidFolders) {
      const fullPath = join(tmpDir, folder.name, STATUS_DIRNAME, "state.json");
      try {
        const stats = statSync(fullPath);
        if (stats.mtimeMs > latestMtime) {
          latestMtime = stats.mtimeMs;
          latestPath = fullPath;
        }
      } catch {
        // Skip files that can't be accessed
      }
    }
  } catch {
    // Discovery failed, return null
  }

  return latestPath;
}

export async function loadState(path: string): Promise<StatuslineState> {
  try {
    const raw = await readFile(path, "utf8");
    const parsed = JSON.parse(raw) as Partial<StatuslineState>;
    if (!parsed || typeof parsed !== "object") {
      return createEmptyState();
    }
    const children =
      parsed.children && typeof parsed.children === "object"
        ? parsed.children
        : {};
    return {
      children: children as Record<string, ChildSessionState>,
      updatedAt:
        typeof parsed.updatedAt === "string"
          ? parsed.updatedAt
          : new Date().toISOString(),
    };
  } catch {
    return createEmptyState();
  }
}

export type StateCallback = (state: StatuslineState) => void;

export async function watchState(
  path: string,
  callback: StateCallback,
): Promise<() => void> {
  let fallbackInterval: ReturnType<typeof setInterval> | null = null;
  let stopped = false;

  const loadAndCallback = async (): Promise<void> => {
    if (stopped) return;
    const state = await loadState(path);
    callback(state);
  };

  try {
    const chokidar = await import("chokidar");
    const watcher = chokidar.watch(path, {
      persistent: true,
      ignoreInitial: false,
      awaitWriteFinish: {
        stabilityThreshold: 100,
        pollInterval: 50,
      },
    });

    watcher.on("change", loadAndCallback);
    watcher.on("unlink", () => {
      if (stopped) return;
      callback(createEmptyState());
    });

    watcher.on("error", () => {
      watcher.close();
      // Fallback to polling
      fallbackInterval = setInterval(loadAndCallback, 2000);
    });

    return () => {
      stopped = true;
      watcher.close();
      if (fallbackInterval) {
        clearInterval(fallbackInterval);
      }
    };
  } catch {
    // chokidar unavailable, use polling fallback
    fallbackInterval = setInterval(loadAndCallback, 2000);
    return () => {
      stopped = true;
      if (fallbackInterval) {
        clearInterval(fallbackInterval);
      }
    };
  }
}