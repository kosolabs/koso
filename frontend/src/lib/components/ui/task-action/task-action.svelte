<script lang="ts">
  import { auth } from "$lib/auth.svelte";
  import type { Koso, Node } from "$lib/dag-table";
  import { Menu, MenuItem, MenuTrigger } from "$lib/kosui/menu";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { unmanagedKinds, type Kind, type YStatus } from "$lib/yproxy";
  import { Bot, Check, CircleCheck, LoaderCircle } from "lucide-svelte";
  import { TaskStatusIcon } from ".";
  import { CircularProgress } from "../../../kosui/progress";
  import { confetti } from "../confetti";
  import { ResponsiveText } from "../responsive-text";

  const statuses: YStatus[] = ["Not Started", "In Progress", "Done"];

  type Props = {
    node: Node;
    koso: Koso;
  };
  let { node, koso }: Props = $props();

  let open = $state(false);
  let statusElement: HTMLElement | undefined = $state();

  let task = $derived(koso.getTask(node.name));
  let progress = $derived(koso.getProgress(task.id));
  let canSetStatus = $derived(
    koso.isEditable(task.id) &&
      !progress.isBlocked() &&
      progress.kind !== "Rollup",
  );
  let canSetKind = $derived(
    koso.isEditable(task.id) &&
      (progress.kind === "Rollup" || progress.kind === "Juggled"),
  );

  function handleOnSelectKind(kind: Kind) {
    if (progress.kind === kind) return;
    koso.setKind(task.id, kind, auth.user);
  }

  function handleOnSelectStatus(status: YStatus) {
    if (progress.status === status) return;
    if (status === "Done") showDoneConfetti();
    koso.setTaskStatus(node, status, auth.user);
  }

  function triggerTitle() {
    if (progress.kind !== "Rollup") {
      return `${progress.status}${progress.lastStatusTime ? ` - ${new Date(progress.lastStatusTime).toLocaleString()}` : ""}`;
    }
    return `${progress.done} of ${progress.total} (${Math.round(
      (progress.done * 100) / progress.total,
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

  function handleKeyDown(event: KeyboardEvent) {
    if (Shortcut.ESCAPE.matches(event)) {
      statusElement?.blur();
      event.stopImmediatePropagation();
    }
  }
</script>

<Menu bind:open>
  {#snippet trigger(menuTriggerProps)}
    <MenuTrigger
      bind:el={statusElement}
      class="rounded-m3 focus:ring-m3-primary flex w-full items-center gap-2 focus-visible:ring-1 focus-visible:outline-hidden"
      title={triggerTitle()}
      aria-label="task-status"
      disabled={!canSetStatus && !canSetKind}
      onkeydown={handleKeyDown}
      {...menuTriggerProps}
    >
      {#if progress.kind === "Rollup"}
        {#if progress.status === "Done"}
          <CircleCheck color="hsl(var(--primary))" />
          <ResponsiveText>Done</ResponsiveText>
        {:else if progress.status === "Not Started"}
          <CircularProgress progress={0} color="hsl(var(--primary))" />
          <ResponsiveText>Not Started</ResponsiveText>
        {:else}
          <CircularProgress
            progress={progress.done / progress.total}
            color="hsl(var(--primary))"
          >
            {Math.round((progress.done * 100) / progress.total)}%
          </CircularProgress>
          <ResponsiveText>In Progress</ResponsiveText>
        {/if}
      {:else}
        <TaskStatusIcon status={progress.status} />
        <ResponsiveText>{progress.status}</ResponsiveText>
      {/if}
    </MenuTrigger>
  {/snippet}
  {#snippet content(menuItemProps)}
    {#if canSetStatus}
      {#each statuses as status (status)}
        <MenuItem
          class="flex items-center gap-2 rounded text-sm"
          onSelect={() => handleOnSelectStatus(status)}
          {...menuItemProps}
        >
          <TaskStatusIcon {status} />
          {status}
        </MenuItem>
      {/each}
    {/if}
    {#if canSetStatus && canSetKind}
      <hr class="my-1" />
    {/if}
    {#if canSetKind}
      {#each unmanagedKinds as kind (kind)}
        <MenuItem
          class="flex items-center gap-2 rounded text-sm"
          onSelect={() => handleOnSelectKind(kind)}
          {...menuItemProps}
        >
          {#if kind === "Rollup"}
            <LoaderCircle class="text-m3-primary" />
          {:else if kind === "Juggled"}
            <Bot class="text-m3-primary" />
          {/if}
          {kind}
          {#if progress.kind === kind}
            <Check class="text-m3-primary ml-auto" size={20} />
          {/if}
        </MenuItem>
      {/each}
    {/if}
  {/snippet}
</Menu>
