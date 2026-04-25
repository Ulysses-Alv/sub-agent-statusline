import { describe, it, expect } from "vitest";
import * as fs from "fs";
import * as path from "path";
import { getStatusCounts } from "../src/stores/state";
import type { StatuslineState } from "../src/stores/state";

// Fixture file paths
const fixturesDir = path.join(__dirname, "fixtures");

function loadFixture(name: string): StatuslineState {
  const filePath = path.join(fixturesDir, name);
  const content = fs.readFileSync(filePath, "utf-8");
  return JSON.parse(content) as StatuslineState;
}

describe("state parser utilities", () => {
  describe("getStatusCounts", () => {
    it("returns zeros for null state", () => {
      const result = getStatusCounts(null);
      expect(result).toEqual({ running: 0, done: 0, error: 0 });
    });

    it("counts running subagents correctly", () => {
      const state: StatuslineState = {
        children: {
          "1": { id: "1", title: "task-1", parentID: "p1", status: "running", color: "yellow", startedAt: "", updatedAt: "" },
          "2": { id: "2", title: "task-2", parentID: "p1", status: "running", color: "yellow", startedAt: "", updatedAt: "" },
          "3": { id: "3", title: "task-3", parentID: "p1", status: "done", color: "green", startedAt: "", updatedAt: "" },
        },
        updatedAt: "",
      };
      const result = getStatusCounts(state);
      expect(result).toEqual({ running: 2, done: 1, error: 0 });
    });

    it("counts done subagents correctly", () => {
      const state: StatuslineState = {
        children: {
          "1": { id: "1", title: "task-1", parentID: "p1", status: "done", color: "green", startedAt: "", updatedAt: "" },
          "2": { id: "2", title: "task-2", parentID: "p1", status: "done", color: "green", startedAt: "", updatedAt: "" },
        },
        updatedAt: "",
      };
      const result = getStatusCounts(state);
      expect(result).toEqual({ running: 0, done: 2, error: 0 });
    });

    it("counts error subagents correctly", () => {
      const state: StatuslineState = {
        children: {
          "1": { id: "1", title: "task-1", parentID: "p1", status: "error", color: "red", startedAt: "", updatedAt: "" },
        },
        updatedAt: "",
      };
      const result = getStatusCounts(state);
      expect(result).toEqual({ running: 0, done: 0, error: 1 });
    });

    it("handles mixed statuses", () => {
      const state: StatuslineState = {
        children: {
          "1": { id: "1", title: "task-1", parentID: "p1", status: "running", color: "yellow", startedAt: "", updatedAt: "" },
          "2": { id: "2", title: "task-2", parentID: "p1", status: "done", color: "green", startedAt: "", updatedAt: "" },
          "3": { id: "3", title: "task-3", parentID: "p1", status: "error", color: "red", startedAt: "", updatedAt: "" },
          "4": { id: "4", title: "task-4", parentID: "p1", status: "done", color: "green", startedAt: "", updatedAt: "" },
        },
        updatedAt: "",
      };
      const result = getStatusCounts(state);
      expect(result).toEqual({ running: 1, done: 2, error: 1 });
    });

    it("handles empty children", () => {
      const state: StatuslineState = {
        children: {},
        updatedAt: "",
      };
      const result = getStatusCounts(state);
      expect(result).toEqual({ running: 0, done: 0, error: 0 });
    });
  });

  describe("fixture loading", () => {
    it("loads empty-state.json correctly", () => {
      const state = loadFixture("empty-state.json");
      expect(state.children).toEqual({});
      expect(state.updatedAt).toBe("2024-01-01T00:00:00.000Z");
    });

    it("loads single-running.json correctly", () => {
      const state = loadFixture("single-running.json");
      expect(Object.keys(state.children)).toHaveLength(1);
      expect(state.children["agent-1"]).toMatchObject({
        id: "agent-1",
        title: "Research Task",
        status: "running",
        color: "yellow",
      });
    });

    it("loads mixed-status.json with all three statuses", () => {
      const state = loadFixture("mixed-status.json");
      expect(Object.keys(state.children)).toHaveLength(3);
      const counts = getStatusCounts(state);
      expect(counts).toEqual({ running: 1, done: 1, error: 1 });
    });

    it("loads with-tokens.json and has token data", () => {
      const state = loadFixture("with-tokens.json");
      expect(state.children["agent-1"].tokens).toEqual({
        input: 15000,
        output: 45000,
        total: 60000,
        contextPercent: 72.5,
      });
    });

    it("throws on malformed.json", () => {
      const filePath = path.join(fixturesDir, "malformed.json");
      const content = fs.readFileSync(filePath, "utf-8");
      expect(() => JSON.parse(content)).toThrow();
    });
  });
});