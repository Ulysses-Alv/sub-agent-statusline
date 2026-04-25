import { describe, it, expect, vi, afterEach } from "vitest";
import { render, cleanup } from "@solidjs/testing-library";
import StatusBar from "../src/components/StatusBar";
import SubagentRow from "../src/components/SubagentRow";
import type { ChildSessionState } from "../src/stores/state";

// Mock the store functions that SubagentRow uses
vi.mock("../src/stores/state", () => ({
  toggleRowExpansion: vi.fn(),
  isRowExpanded: vi.fn(() => false),
}));

describe("UI Components", () => {
  describe("StatusBar", () => {
    it("renders running, done, and error counts", () => {
      const { container } = render(() => (
        <StatusBar running={2} done={3} error={1} />
      ));

      expect(container.textContent).toContain("2 running");
      expect(container.textContent).toContain("3 done");
      expect(container.textContent).toContain("1 error");
    });

    it("renders correct status dots", () => {
      const { container } = render(() => (
        <StatusBar running={0} done={0} error={0} />
      ));

      const runningDots = container.querySelectorAll(".status-dot.running");
      const doneDots = container.querySelectorAll(".status-dot.done");
      const errorDots = container.querySelectorAll(".status-dot.error");

      expect(runningDots.length).toBe(1);
      expect(doneDots.length).toBe(1);
      expect(errorDots.length).toBe(1);
    });

    it("handles zero counts", () => {
      const { container } = render(() => (
        <StatusBar running={0} done={0} error={0} />
      ));

      expect(container.textContent).toContain("0 running");
      expect(container.textContent).toContain("✓ 0 done");
      expect(container.textContent).toContain("✕ 0 error");
    });

    it("renders single digit counts correctly", () => {
      const { container } = render(() => (
        <StatusBar running={5} done={10} error={3} />
      ));

      expect(container.textContent).toContain("5 running");
      expect(container.textContent).toContain("10 done");
      expect(container.textContent).toContain("3 error");
    });
  });

  describe("SubagentRow", () => {
    const createMockChild = (overrides: Partial<ChildSessionState> = {}): ChildSessionState => ({
      id: "test-1",
      title: "Test Task",
      parentID: "parent-1",
      status: "running",
      color: "yellow",
      startedAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      ...overrides,
    });

    afterEach(() => {
      cleanup();
    });

    it("renders title when collapsed", () => {
      const child = createMockChild();
      const { container } = render(() => <SubagentRow child={child} />);

      expect(container.textContent).toContain("Test Task");
    });

    it("renders elapsed time when elapsedMs provided", () => {
      const child = createMockChild({ elapsedMs: 65000 }); // 1m 5s
      const { container } = render(() => <SubagentRow child={child} />);

      expect(container.textContent).toContain("1m 5s");
    });

    it("shows correct status dot class for running", () => {
      const child = createMockChild({ status: "running", color: "yellow" });
      const { container } = render(() => <SubagentRow child={child} />);

      const dot = container.querySelector(".status-dot.running");
      expect(dot).not.toBeNull();
    });

    it("shows correct status dot class for done", () => {
      const child = createMockChild({ status: "done", color: "green" });
      const { container } = render(() => <SubagentRow child={child} />);

      const dot = container.querySelector(".status-dot.done");
      expect(dot).not.toBeNull();
    });

    it("shows correct status dot class for error", () => {
      const child = createMockChild({ status: "error", color: "red" });
      const { container } = render(() => <SubagentRow child={child} />);

      const dot = container.querySelector(".status-dot.error");
      expect(dot).not.toBeNull();
    });

    it("renders different titles correctly", () => {
      const child = createMockChild({ title: "Build Project" });
      const { container } = render(() => <SubagentRow child={child} />);

      expect(container.textContent).toContain("Build Project");
    });

    it("handles long elapsed times", () => {
      const child = createMockChild({ elapsedMs: 3661000 }); // 1h 1m 1s
      const { container } = render(() => <SubagentRow child={child} />);

      expect(container.textContent).toContain("1h 1m");
    });
  });
});