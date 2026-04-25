import { Component, For, Show } from "solid-js";
import type { ChildSessionState } from "../stores/state";
import SubagentRow from "./SubagentRow";

interface SubagentListProps {
  children: Record<string, ChildSessionState>;
}

const SubagentList: Component<SubagentListProps> = (props) => {
  const childList = () => Object.values(props.children);

  return (
    <div class="subagent-list">
      <Show
        when={childList().length > 0}
        fallback={<div class="empty-state">No subagents running</div>}
      >
        <For each={childList()}>
          {(child) => <SubagentRow child={child} />}
        </For>
      </Show>
    </div>
  );
};

export default SubagentList;