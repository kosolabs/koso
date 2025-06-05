<script lang="ts">
  import { getAuthContext } from "$lib/auth.svelte";
  import { getRegistryContext } from "$lib/components/ui/command-palette";
  import { toast } from "$lib/components/ui/sonner";
  import type { Koso } from "$lib/dag-table/koso.svelte";
  import { getDialoguerContext } from "$lib/kosui/dialog";
  import {
    Menu,
    MenuContent,
    MenuDivider,
    MenuItem,
    MenuTrigger,
  } from "$lib/kosui/menu";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { YTaskProxy, type Status } from "$lib/yproxy";
  import {
    CalendarDays,
    Check,
    CircleCheck,
    ClipboardCheck,
    IterationCcw,
    LoaderCircle,
  } from "lucide-svelte";
  import { TaskStatusIcon } from ".";
  import { CircularProgress } from "../../../kosui/progress";
  import { confetti } from "../confetti";
  import { ResponsiveText } from "../responsive-text";

  type Props = {
    koso: Koso;
    task: YTaskProxy;
    inboxView: boolean;
  };
  let { koso, task, inboxView }: Props = $props();

  const command = getRegistryContext();
  const auth = getAuthContext();
  const dialog = getDialoguerContext();

  let open = $state(false);
  let statusElement: HTMLElement | undefined = $state();

  let progress = $derived(koso.getProgress(task.id));
  let canSetStatus = $derived(koso.isEditable(task.id) && !task.isRollup());
  let canSetKind = $derived(koso.isEditable(task.id));
  let statuses: Status[] = ["Not Started", "In Progress", "Done", "Blocked"];
  let deadline = $derived(
    task.deadline
      ? new Date(task.deadline).toISOString().split("T")[0]
      : undefined,
  );

  function handleSelectKindTask() {
    koso.setKind(task.id, "Task");
  }

  function handleSelectKindRollup() {
    task.deadline = null;
    koso.setKind(task.id, "Rollup");
  }

  async function handleSelectKindIteration() {
    const date = await dialog.input({
      props: { type: "date", value: deadline },
      icon: CalendarDays,
      title: "Iteration End Date",
      message: "Select a date to mark the end of this iteration.",
    });
    if (date) {
      koso.doc.transact(() => {
        task.deadline = Date.parse(date);
        koso.setKind(task.id, "Rollup");
      });
    }
  }

  function handleOnSelectStatus(status: Status) {
    if (status === "Done") {
      showDoneConfetti();
      koso.setTaskStatus(task.id, "Done", auth.user);
      if (inboxView) {
        toast.success("🚀 Great work! Task complete!");
      }
    } else if (task.children.length === 0 && status === "Blocked") {
      command.call("Block");
    } else {
      koso.setTaskStatus(task.id, status, auth.user);
    }
  }

  function triggerTitle() {
    if (!task.isRollup()) {
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

{#snippet responsiveTextWithDeadline(text: string)}
  <ResponsiveText class="flex flex-col items-start">
    <div>{text}</div>
    {#if deadline}
      <div class="text-xs">{deadline}</div>
    {/if}
  </ResponsiveText>
{/snippet}

<Menu bind:open bind:el={statusElement}>
  <MenuTrigger
    class="focus:ring-m3-primary flex w-full items-center gap-2 focus-visible:ring-1 focus-visible:outline-hidden"
    title={triggerTitle()}
    aria-label="task-status"
    disabled={!canSetStatus && !canSetKind}
    onkeydown={handleKeyDown}
  >
    {#if task.isRollup()}
      {#if progress.status === "Done"}
        <CircleCheck class="text-m3-primary" />
        {@render responsiveTextWithDeadline("Done")}
      {:else if progress.status === "Not Started"}
        <CircularProgress progress={0} class="text-m3-primary" />
        {@render responsiveTextWithDeadline("Not Started")}
      {:else}
        <CircularProgress
          progress={progress.done / progress.total}
          class="text-m3-primary"
        >
          {Math.round((progress.done * 100) / progress.total)}%
        </CircularProgress>
        {@render responsiveTextWithDeadline("In Progress")}
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
      <MenuItem
        class="flex items-center gap-2 rounded text-sm"
        onSelect={() => handleSelectKindIteration()}
      >
        <IterationCcw class="text-m3-primary" />
        Iteration...
        {#if task.isRollup() && deadline}
          <Check class="text-m3-primary ml-auto" size={20} />
        {/if}
      </MenuItem>
      <MenuItem
        class="flex items-center gap-2 rounded text-sm"
        onSelect={() => handleSelectKindRollup()}
      >
        <LoaderCircle class="text-m3-primary" />
        Rollup
        {#if task.isRollup() && !deadline}
          <Check class="text-m3-primary ml-auto" size={20} />
        {/if}
      </MenuItem>
      <MenuItem
        class="flex items-center gap-2 rounded text-sm"
        onSelect={() => handleSelectKindTask()}
      >
        <ClipboardCheck class="text-m3-primary" />
        Task
        {#if !task.isRollup()}
          <Check class="text-m3-primary ml-auto" size={20} />
        {/if}
      </MenuItem>
    {/if}
  </MenuContent>
</Menu>
