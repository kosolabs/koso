<script module lang="ts">
  export type TaskActionType = {
    showDoneConfetti(): void;
  };
</script>

<script lang="ts">
  import { auth } from "$lib/auth.svelte";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import type { Koso, Node } from "$lib/dag-table";
  import { Shortcut } from "$lib/shortcuts";
  import { unmanagedKinds, type Kind, type Status } from "$lib/yproxy";
  import { Bot, CircleCheck, LoaderCircle } from "lucide-svelte";
  import { tick } from "svelte";
  import { TaskStatusIcon } from ".";
  import { CircularProgress } from "../../../kosui/progress";
  import { confetti } from "../confetti";
  import { ResponsiveText } from "../responsive-text";

  const statuses: Status[] = ["Not Started", "In Progress", "Done"];

  type Props = {
    node: Node;
    koso: Koso;
  };
  let { node, koso }: Props = $props();

  let open = $state(false);
  let statusElement: HTMLElement | null = $state(null);

  let task = $derived(koso.getTask(node.name));
  let canSetStatus = $derived(
    koso.isEditable(task.id) && task.children.length === 0,
  );
  let canSetKind = $derived(
    koso.isEditable(task.id) && task.children.length > 0,
  );
  let rollupProgress = $derived(
    task.children.length > 0 || task.kind === "Rollup"
      ? koso.getProgress(task.id)
      : null,
  );

  function handleOnSelectKind(kind: Kind) {
    koso.setKind(task.id, kind);
  }

  function handleOnSelectStatus(status: Status) {
    if (status === "Done") showDoneConfetti();
    koso.setTaskStatus(node, status, auth.user);
  }

  function triggerTitle() {
    if (!rollupProgress) {
      return `${task.status || "Not Started"}${task.statusTime ? ` - ${new Date(task.statusTime).toLocaleString()}` : ""}`;
    }
    return `${rollupProgress.done} of ${rollupProgress.total} (${Math.round(
      (rollupProgress.done * 100) / rollupProgress.total,
    )}%)`;
  }

  /**
   * Triggers the confetti animation when a task's status set to Done. The
   * confetti's position is relative to the dropdown trigger in this component.
   * Thus, this function is exported for use in other flows that modify task
   * status.
   */
  export function showDoneConfetti() {
    confetti.add(getStatusPosition());
  }

  function getStatusPosition(): DOMRect {
    if (!statusElement) throw new Error("Status element is undefined");
    return statusElement.getBoundingClientRect();
  }
</script>

<DropdownMenu.Root
  bind:open={
    () => open,
    (newOpen) => {
      koso.selected = node;
      tick().then(() => (open = newOpen));
    }
  }
>
  <DropdownMenu.Trigger
    bind:ref={statusElement}
    class="flex items-center gap-2"
    title={triggerTitle()}
    disabled={!canSetStatus && !canSetKind}
    aria-label="task-status"
  >
    {#if rollupProgress}
      {#if rollupProgress.done === rollupProgress.total}
        <CircleCheck color="hsl(var(--primary))" />
        <ResponsiveText>Done</ResponsiveText>
      {:else if rollupProgress.done === 0 && rollupProgress.inProgress == 0}
        <CircularProgress progress={0} color="hsl(var(--primary))" />
        <ResponsiveText>Not Started</ResponsiveText>
      {:else}
        <CircularProgress
          progress={rollupProgress.done / rollupProgress.total}
          color="hsl(var(--primary))"
        >
          {Math.round((rollupProgress.done * 100) / rollupProgress.total)}%
        </CircularProgress>
        <ResponsiveText>In Progress</ResponsiveText>
      {/if}
    {:else}
      <TaskStatusIcon status={task.status} />
      <ResponsiveText>{task.status || "Not Started"}</ResponsiveText>
    {/if}
  </DropdownMenu.Trigger>
  <div
    role="none"
    onkeydown={(event) => {
      if (Shortcut.CANCEL.matches(event)) {
        open = false;
      }
      event.stopPropagation();
    }}
  >
    <DropdownMenu.Content
      portalProps={{ disabled: true }}
      preventScroll={false}
    >
      {#if canSetStatus}
        {#each statuses as status}
          <DropdownMenu.Item
            class="flex items-center gap-2 rounded p-2"
            onSelect={() => handleOnSelectStatus(status)}
          >
            <TaskStatusIcon {status} />
            {status}
          </DropdownMenu.Item>
        {/each}
      {/if}
      {#if canSetKind}
        {#each unmanagedKinds as kind}
          <DropdownMenu.Item
            class="flex items-center gap-2 rounded p-2"
            onSelect={() => handleOnSelectKind(kind)}
          >
            {#if kind === "Rollup"}
              <LoaderCircle />
            {:else if kind === "Juggled"}
              <Bot />
            {/if}
            {kind}
          </DropdownMenu.Item>
        {/each}
      {/if}
    </DropdownMenu.Content>
  </div>
</DropdownMenu.Root>
