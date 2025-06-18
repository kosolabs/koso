<script lang="ts">
  import { getAuthContext } from "$lib/auth.svelte";
  import { getRegistryContext } from "$lib/components/ui/command-palette";
  import { toast } from "$lib/components/ui/sonner";
  import type { Koso } from "$lib/dag-table/koso.svelte";
  import { Action } from "$lib/kosui/command";
  import { getDialoguerContext } from "$lib/kosui/dialog";
  import { Menu, MenuActions, MenuContent, MenuTrigger } from "$lib/kosui/menu";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { YTaskProxy, type Status } from "$lib/yproxy";
  import {
    CalendarDays,
    Circle,
    CircleCheck,
    CircleDotDashed,
    CircleFadingArrowUp,
    ClipboardCheck,
    IterationCcw,
    LoaderCircle,
    PauseOctagon,
    WandSparkles,
  } from "@lucide/svelte";
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
  let deadline = $derived(
    task.deadline
      ? new Date(task.deadline).toISOString().split("T")[0]
      : undefined,
  );

  let actions: Action[] = $derived(
    [
      new Action({
        id: "Status.NotStarted",
        callback: () => handleOnSelectStatus("Not Started"),
        category: "Status",
        name: "Not Started",
        description: "Set status of task to Not Started",
        icon: Circle,
        enabled: () => task.isTask(),
        selected: () => progress.status === "Not Started",
      }),
      new Action({
        id: "Status.Ready",
        callback: () => handleOnSelectStatus("Ready"),
        category: "Status",
        name: "Ready",
        description: "Set status of task to Ready",
        icon: CircleDotDashed,
        enabled: () => task.isTask(),
        selected: () => progress.status === "Ready",
      }),
      new Action({
        id: "Status.InProgress",
        callback: () => handleOnSelectStatus("In Progress"),
        category: "Status",
        name: "In Progress",
        description: "Set status of task to In Progress",
        icon: CircleFadingArrowUp,
        enabled: () => task.isTask(),
        selected: () => progress.status === "In Progress",
      }),
      new Action({
        id: "Status.Done",
        callback: () => handleOnSelectStatus("Done"),
        category: "Status",
        name: "Done",
        description: "Set status of task to Done",
        icon: CircleCheck,
        enabled: () => task.isTask(),
        selected: () => progress.status === "Done",
      }),
      new Action({
        id: "Status.Blocked",
        callback: () => handleOnSelectStatus("Blocked"),
        category: "Status",
        name: "Blocked",
        description: "Set status of task to Blocked",
        icon: PauseOctagon,
        enabled: () => task.isTask(),
        selected: () => progress.status === "Blocked",
      }),

      new Action({
        id: "Kind.Auto",
        callback: () => handleSelectKindAuto(),
        category: "Kind",
        name: `Auto (${task.autoType()})`,
        description: "Set kind of task to Auto",
        icon: WandSparkles,
        enabled: () => !task.isManaged(),
        selected: () => currentMenuSelection() === "Auto",
      }),
      new Action({
        id: "Kind.Task",
        callback: () => handleSelectKindTask(),
        category: "Kind",
        name: "Task",
        description: "Set kind of task to Task",
        icon: ClipboardCheck,
        enabled: () => !task.isManaged(),
        selected: () => currentMenuSelection() === "Task",
      }),
      new Action({
        id: "Kind.Rollup",
        callback: () => handleSelectKindRollup(),
        category: "Kind",
        name: "Rollup",
        description: "Set kind of task to Rollup",
        icon: LoaderCircle,
        enabled: () => !task.isManaged(),
        selected: () => currentMenuSelection() === "Rollup",
      }),
      new Action({
        id: "Kind.Iteration",
        callback: () => handleSelectKindIteration(),
        category: "Kind",
        name: "Iteration...",
        description: "Set kind of task to Iteration",
        icon: IterationCcw,
        enabled: () => !task.isManaged(),
        selected: () => currentMenuSelection() === "Iteration",
      }),
    ].filter((action) => action.enabled()),
  );

  function handleSelectKindTask() {
    koso.setKind(task.id, "Task");
  }

  function handleSelectKindRollup() {
    koso.doc.transact(() => {
      task.deadline = null;
      koso.setKind(task.id, "Rollup");
    });
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

  async function handleSelectKindAuto() {
    koso.doc.transact(() => {
      task.deadline = null;
      koso.setKind(task.id, null);
    });
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

  function currentMenuSelection():
    | "Iteration"
    | "Rollup"
    | "Task"
    | "Auto"
    | null {
    if (task.isAuto()) {
      return "Auto";
    }
    if (task.isIteration()) {
      return "Iteration";
    }
    if (task.isRollup()) {
      return "Rollup";
    }
    return task.kind === "Task" ? "Task" : null;
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
    disabled={task.isManaged()}
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
    <MenuActions {actions} />
  </MenuContent>
</Menu>
