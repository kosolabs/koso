<script lang="ts">
  import { auth } from "$lib/auth.svelte";
  import { toast } from "$lib/components/ui/sonner";
  import type { Koso, Node } from "$lib/dag-table";
  import { command } from "$lib/kosui/command";
  import {
    Menu,
    MenuContent,
    MenuDivider,
    MenuItem,
    MenuTrigger,
  } from "$lib/kosui/menu";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { unmanagedKinds, type Kind, type Status } from "$lib/yproxy";
  import { Bot, Check, CircleCheck, LoaderCircle } from "lucide-svelte";
  import { TaskStatusIcon } from ".";
  import { CircularProgress } from "../../../kosui/progress";
  import { confetti } from "../confetti";
  import { ResponsiveText } from "../responsive-text";

  type Props = {
    node: Node;
    koso: Koso;
    inboxView: boolean;
  };
  let { node, koso, inboxView }: Props = $props();

  let open = $state(false);
  let statusElement: HTMLElement | undefined = $state();

  let task = $derived(koso.getTask(node.name));
  let progress = $derived(koso.getProgress(task.id));
  let canSetStatus = $derived(
    koso.isEditable(task.id) && progress.kind !== "Rollup",
  );
  let canSetKind = $derived(
    koso.isEditable(task.id) &&
      (progress.kind === "Rollup" ||
        (progress.kind === "Task" && progress.childrenStatus != null)),
  );
  let statuses: Status[] = ["Not Started", "In Progress", "Done", "Blocked"];

  function handleOnSelectKind(kind: Kind) {
    if (progress.kind === kind) return;
    koso.setKind(task.id, kind);
  }

  function handleOnSelectStatus(status: Status) {
    if (status === "Done") {
      showDoneConfetti();
      koso.setTaskStatus(node, "Done", auth.user);
      if (inboxView) {
        toast.success("🚀 Great work! Task complete!");
      }
    } else if (task.children.length === 0 && status === "Blocked") {
      command.call("Block task on...");
    } else {
      koso.setTaskStatus(node, status, auth.user);
    }
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

<Menu bind:open bind:el={statusElement}>
  <MenuTrigger
    class="focus:ring-m3-primary flex w-full items-center gap-2 focus-visible:ring-1 focus-visible:outline-hidden"
    title={triggerTitle()}
    aria-label="task-status"
    disabled={!canSetStatus && !canSetKind}
    onkeydown={handleKeyDown}
  >
    {#if progress.kind === "Rollup"}
      {#if progress.status === "Done"}
        <CircleCheck class="text-m3-primary" />
        <ResponsiveText>Done</ResponsiveText>
      {:else if progress.status === "Not Started"}
        <CircularProgress progress={0} class="text-m3-primary" />
        <ResponsiveText>Not Started</ResponsiveText>
      {:else}
        <CircularProgress
          progress={progress.done / progress.total}
          class="text-m3-primary"
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
  <MenuContent>
    {#if canSetStatus}
      {#each statuses as status (status)}
        <MenuItem
          class="flex items-center gap-2 rounded text-sm"
          onSelect={() => handleOnSelectStatus(status)}
        >
          <TaskStatusIcon {status} />
          {status}
          {#if progress.status === status}
            <Check class="text-m3-primary ml-auto" size={20} />
          {/if}
        </MenuItem>
      {/each}
    {/if}
    {#if canSetStatus && canSetKind}
      <MenuDivider />
    {/if}
    {#if canSetKind}
      {#each unmanagedKinds as kind (kind)}
        <MenuItem
          class="flex items-center gap-2 rounded text-sm"
          onSelect={() => handleOnSelectKind(kind)}
        >
          {#if kind === "Rollup"}
            <LoaderCircle class="text-m3-primary" />
          {:else if kind === "Task"}
            <Bot class="text-m3-primary" />
          {/if}
          {kind}
          {#if progress.kind === kind}
            <Check class="text-m3-primary ml-auto" size={20} />
          {/if}
        </MenuItem>
      {/each}
    {/if}
  </MenuContent>
</Menu>
