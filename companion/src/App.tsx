import { Component, Show, onMount, onCleanup, createMemo } from "solid-js";
import { pids, setPids, setSelectedPid, selectedPid, state, setState, watching, setWatching, error, setError, loading, setLoading, getStatusCounts, clearState } from "./stores/state";
import { discoverPids, startWatching, stopWatching, onStateUpdated, onPidsChanged, readStateNow } from "./lib/tauri";
import StatusBar from "./components/StatusBar";
import PidSelector from "./components/PidSelector";
import SubagentList from "./components/SubagentList";
import "./styles/app.css";

const POLL_INTERVAL_MS = 2000;
const DISCOVER_INTERVAL_MS = 5000;

const App: Component = () => {
  let unlistenStateUpdated: (() => void) | undefined;
  let unlistenPidsChanged: (() => void) | undefined;
  let pollTimer: ReturnType<typeof setInterval> | undefined;
  let discoverTimer: ReturnType<typeof setInterval> | undefined;
  let currentPid: number | null = null;

  const statusCounts = createMemo(() => getStatusCounts(state()));

  const pollState = async (pid?: number) => {
    const targetPid = pid || selectedPid();
    if (!targetPid) return;
    currentPid = targetPid;
    try {
      const newState = await readStateNow(targetPid);
      console.log("[pollState] readStateNow(", targetPid, ") =", newState ? Object.keys(newState.children).length + " children" : "null");
      if (newState) {
        setState(newState);
        setError(null);
      }
    } catch (e) {
      console.log("[pollState] error:", e);
    }
  };

  const runDiscover = async () => {
    try {
      const discoveredPids = await discoverPids();
      console.log("[runDiscover] discoverPids() =", discoveredPids);
      setPids(discoveredPids);

      if (discoveredPids.length > 0 && !selectedPid()) {
        const pid = discoveredPids[0].pid;
        console.log("[runDiscover] Selecting PID:", pid);
        setSelectedPid(pid);
        await startWatching(pid);
        setWatching(true);
        currentPid = pid;
        if (pollTimer) clearInterval(pollTimer);
        pollTimer = setInterval(() => pollState(), POLL_INTERVAL_MS);
        pollState(pid);
      }
    } catch (e) {
      console.log("[runDiscover] error:", e);
    }
  };

  const handlePidSelect = async (pid: number) => {
    if (watching()) {
      await stopWatching();
      setWatching(false);
    }
    setSelectedPid(pid);
    await startWatching(pid);
    setWatching(true);
    currentPid = pid;
    if (pollTimer) clearInterval(pollTimer);
    pollTimer = setInterval(() => pollState(), POLL_INTERVAL_MS);
    pollState(pid);
  };

  const handleClear = async () => {
    clearState();
    if (currentPid) {
      pollState(currentPid);
    }
  };

  onMount(async () => {
    setLoading(true);
    setError(null);

    try {
      await runDiscover();

      unlistenStateUpdated = await onStateUpdated((newState) => {
        if (newState) {
          setState(newState);
        }
      });

      unlistenPidsChanged = await onPidsChanged((newPids) => {
        setPids(newPids);
      });

      // Periodically re-discover PIDs (catches late-starting state.json)
      discoverTimer = setInterval(runDiscover, DISCOVER_INTERVAL_MS);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  });

  onCleanup(async () => {
    if (pollTimer) clearInterval(pollTimer);
    if (discoverTimer) clearInterval(discoverTimer);
    if (watching()) {
      await stopWatching();
    }
    unlistenStateUpdated?.();
    unlistenPidsChanged?.();
  });

  return (
    <div class="app">
      <div style="background:#000;color:#0f0;font-size:10px;padding:4px;margin-bottom:4px;font-family:monospace">
        PIDs: {pids().map(p => p.pid).join(', ') || 'none'} | sel: {selectedPid() || 'none'} | state: {state() ? Object.keys(state()!.children).length + " childs" : "null"}
      </div>

      <Show when={error()}>
        <div class="error-state">{error()}</div>
      </Show>

      <Show when={loading()}>
        <div class="loading-state">Loading...</div>
      </Show>

      <Show when={!error() && !loading()}>
        <Show when={pids().length > 1}>
          <PidSelector onSelect={handlePidSelect} />
        </Show>

        <StatusBar
          running={statusCounts().running}
          done={statusCounts().done}
          error={statusCounts().error}
          onClear={handleClear}
        />

        <Show when={state()}>
          <SubagentList children={state()!.children} />
        </Show>

        <Show when={!state() && pids().length === 0}>
          <div class="empty-state">
            No sub-agent processes found.<br />
            Start a sub-agent with statusline to see it here.
          </div>
        </Show>
      </Show>

      <div class="version-footer">v0.1.0</div>
    </div>
  );
};

export default App;