<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import type { Koso } from "$lib/dag-table";
  import {
    unmanagedKinds,
    type Kind,
    type Status,
    type YTaskProxy,
  } from "$lib/yproxy";
  import { Bot, CircleCheck, LoaderCircle } from "lucide-svelte";
  import { TaskStatusIcon } from ".";
  import { CircularProgress } from "../circular-progress";
  import { ResponsiveText } from "../responsive-text";
  import { tick } from "svelte";
  import { Shortcut } from "$lib/shortcuts";

  const statuses: Status[] = ["Not Started", "In Progress", "Done"];

  type Props = {
    task: YTaskProxy;
    koso: Koso;
    onOpenChange?: (open: boolean) => void;
    onSelectKind?: (kind: Kind) => void;
    onSelectStatus?: (status: Status) => void;
  };
  const { task, koso, onOpenChange, onSelectKind, onSelectStatus }: Props =
    $props();

  let open = $state(false);
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
    onSelectKind?.(kind);
  }

  function handleOnSelectStatus(status: Status) {
    onSelectStatus?.(status);
  }

  function triggerTitle() {
    if (!rollupProgress) {
      return `${task.status || "Not Started"}${task.statusTime ? ` - ${new Date(task.statusTime).toLocaleString()}` : ""}`;
    }
    return `${rollupProgress.done} of ${rollupProgress.total} (${Math.round(
      (rollupProgress.done * 100) / rollupProgress.total,
    )}%)`;
  }
</script>

<DropdownMenu.Root
  bind:open={
    () => open,
    (newOpen) => {
      onOpenChange?.(newOpen);
      tick().then(() => (open = newOpen));
    }
  }
>
  <DropdownMenu.Trigger
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
