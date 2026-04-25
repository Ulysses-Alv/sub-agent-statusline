export type ChildStatus = "running" | "done" | "error";

export interface ChildTokenState {
  input?: number;
  output?: number;
  total?: number;
  contextPercent?: number;
}

export interface ChildSessionState {
  id: string;
  title: string;
  parentID: string;
  messageID?: string;
  source?: "session" | "subtask" | "tool";
  status: ChildStatus;
  color: "yellow" | "green" | "red";
  startedAt: string;
  updatedAt: string;
  endedAt?: string;
  elapsedMs?: number;
  tokens?: ChildTokenState;
}

export interface StatuslineState {
  children: Record<string, ChildSessionState>;
  updatedAt: string;
}

export interface StatusCounts {
  running: number;
  done: number;
  error: number;
}
