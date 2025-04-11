<script lang="ts">
  import { replaceState } from "$app/navigation";
  import { auth, type User } from "$lib/auth.svelte";
  import { command, type ActionID } from "$lib/components/ui/command-palette";
  import KosoLogo from "$lib/components/ui/koso-logo/koso-logo.svelte";
  import { toast } from "$lib/components/ui/sonner";
  import { Action } from "$lib/kosui/command";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { CANCEL } from "$lib/shortcuts";
  import {
    Cable,
    Check,
    CircleX,
    OctagonX,
    Pencil,
    Redo,
    SquarePen,
    StepBack,
    StepForward,
    Undo,
    UserRoundPlus,
  } from "lucide-svelte";
  import { onMount, setContext } from "svelte";
  import { flip } from "svelte/animate";
  import { getInboxContext, type Koso } from ".";
  import MarkdownEditor from "./markdown-editor.svelte";
  import TaskRow from "./task-row.svelte";
  import Toolbar from "./toolbar.svelte";

  type Props = {
    koso: Koso;
    users: User[];
  };
  const { koso, users }: Props = $props();

  const inbox = getInboxContext();

  const rows: { [key: string]: TaskRow } = {};

  function getRow(id: string) {
    const maybeRow = rows[id];
    if (!maybeRow) {
      throw new Error(`Row doesn't exist for task ${id}`);
    }
    return maybeRow;
  }

  function toggleStatus() {
    if (!inbox.selected) return;

    const task = koso.getTask(inbox.selected.id);
    let progress = koso.getProgress(task.id);
    if (progress.kind === "Rollup") {
      toast.warning(
        "Cannot change the status of a Rollup task. Change the status of the task's children instead.",
      );
      return;
    }

    switch (progress.status) {
      case "Done":
        return;
      case "Blocked":
        koso.setTaskStatus(task.id, "Not Started", auth.user);
        return;
      case "In Progress": {
        koso.setTaskStatus(task.id, "Done", auth.user);
        toast.success("🚀 Great work! Task complete!");
        break;
      }
      case "Not Started":
        koso.setTaskStatus(task.id, "In Progress", auth.user);
        break;
      default:
        throw new Error(`Unhandled status ${task.yStatus}`);
    }
  }

  function edit() {
    if (!inbox.selected) return;
    getRow(inbox.selected.id).edit(true);
  }

  function unselect() {
    inbox.selected = undefined;
  }

  function selectNext() {
    if (koso.tasks.length > 0) {
      if (inbox.selected) {
        const selectedIndex = koso.getTaskIndex(inbox.selected.id);
        if (selectedIndex < 0) {
          inbox.selected = undefined;
        } else {
          const index = Math.min(selectedIndex + 1, koso.tasks.length - 1);
          inbox.selected = koso.tasks[index];
        }
      } else {
        inbox.selected = koso.tasks[0];
      }
    }
  }

  function selectPrev() {
    if (koso.tasks.length > 0) {
      if (inbox.selected) {
        const selectedIndex = koso.getTaskIndex(inbox.selected.id);
        if (selectedIndex < 0) {
          inbox.selected = undefined;
        } else {
          const index = Math.max(selectedIndex - 1, 0);
          inbox.selected = koso.tasks[index];
        }
      } else {
        inbox.selected = koso.tasks[koso.tasks.length - 1];
      }
    }
  }

  function undo() {
    inbox.undo();
  }

  function redo() {
    inbox.redo();
  }

  function linkTask() {
    if (!inbox.selected) return;
    getRow(inbox.selected.id).linkPanel(true, "link");
  }

  function blockTask() {
    if (!inbox.selected) return;
    getRow(inbox.selected.id).linkPanel(true, "block");
  }

  const undoAction = new Action({
    id: "Undo",
    callback: undo,
    icon: Undo,
    shortcut: new Shortcut({ key: "z", meta: true }),
  });

  const redoAction = new Action({
    id: "Redo",
    callback: redo,
    icon: Redo,
    shortcut: new Shortcut({ key: "z", meta: true, shift: true }),
  });

  const actions: Action<ActionID>[] = [
    new Action({
      id: "Next",
      callback: selectNext,
      description: "Select next task",
      icon: StepForward,
      shortcut: new Shortcut({ key: "ArrowDown" }),
    }),
    new Action({
      id: "Previous",
      callback: selectPrev,
      description: "Select previous task",
      icon: StepBack,
      shortcut: new Shortcut({ key: "ArrowUp" }),
    }),
    new Action({
      id: "Edit",
      callback: edit,
      description: "Edit the current task",
      icon: Pencil,
      shortcut: new Shortcut({ key: "Enter" }),
      enabled: () => !!inbox.selected && koso.isEditable(inbox.selected.id),
    }),
    new Action({
      id: "Clear",
      callback: unselect,
      description: "Clear the current selection",
      icon: CircleX,
      shortcut: CANCEL,
    }),
    undoAction,
    redoAction,
    new Action({
      id: "ToggleTaskStatus",
      callback: toggleStatus,
      title: "Toggle Task Status",
      description: "Toggle the task status to In Progress or Done",
      icon: Check,
      shortcut: new Shortcut({ key: " " }),
      enabled: () => !!inbox.selected && koso.isEditable(inbox.selected.id),
    }),
    new Action({
      id: "Link",
      callback: linkTask,
      title: "Link task to...",
      description: "Link current task to another task",
      icon: Cable,
      enabled: () => !!inbox.selected,
    }),
    new Action({
      id: "Block",
      callback: blockTask,
      title: "Block task on...",
      description: "Block current task to another task",
      icon: OctagonX,
      enabled: () => !!inbox.selected,
      shortcut: new Shortcut({ key: "/", meta: true }),
    }),
  ];

  onMount(async () => {
    const url = new URL(window.location.href);
    const taskId = url.searchParams.get("taskId");
    if (taskId) {
      await koso.synced;
      url.searchParams.delete("taskId");
      replaceState(url, {});
      inbox.select(taskId);
    }
  });

  setContext<Koso>("koso", koso);

  onMount(() => {
    return command.register(...actions);
  });

  // This effect selects a new task when the
  // selected task no longer exists. For example, when
  // the user marks a task as done or blocked in the inbox
  // or a different user deletes the user's currently selected task.
  $effect(() => {
    const selected = inbox.selectedRaw;
    const task = selected.task;
    const index = selected.index;

    if (!task) {
      return;
    }

    const currentIndex = koso.getTaskIndex(task.id);
    if (currentIndex !== -1) {
      // The task still exists. Make sure the stashed index still matches.
      if (!index || index !== currentIndex) {
        console.debug(
          `Refreshing selected index for task ${task.id} at prior index ${index}`,
        );
        inbox.selected = task;
      }
      return;
    }

    // The selected task no longer exists. Select the
    // task at the same index or the one at the end of the list.
    // The first task is not selectable.
    if (koso.tasks.length > 1) {
      console.debug(`Task ${task.id} no longer exists. Selecting new task.`);
      if (!index || index >= koso.tasks.length) {
        inbox.selected = koso.tasks[koso.tasks.length - 1];
      } else {
        inbox.selected = koso.tasks[index];
      }
    } else {
      console.debug(`Task ${task.id} no longer exists. Clearing selection.`);
      inbox.selected = undefined;
    }
  });
</script>

<Toolbar actions={[undoAction, redoAction]}>
  {#await koso.synced then}
    {#if koso.tasks.length > 1}
      <MarkdownEditor taskId={inbox.selected?.id} detailPanelRenderer={inbox} />

      <table class="w-full border-separate border-spacing-0 rounded-md border">
        <thead class="text-left text-xs font-bold uppercase">
          <tr>
            <th class="w-32 p-2">ID</th>
            {#if koso.debug}
              <th class="border-l p-2">UUID</th>
            {/if}
            <th class="border-l p-2">
              <SquarePen class="h-4 md:hidden" />
              <div class="max-md:hidden">Status</div></th
            >
            <th class="border-l p-2">Name</th>
            <th class="p-2"></th>
            <th class="border-l p-2">
              <UserRoundPlus class="h-4 md:hidden" />
              <div class="max-md:hidden">Assignee</div>
            </th>
            <th class="relative m-0 w-0 p-0"></th>
          </tr>
        </thead>

        {#each [...koso.tasks].slice(1) as task, index (task.id)}
          <tbody animate:flip={{ duration: 250 }}>
            <!-- eslint-disable-next-line svelte/no-unused-svelte-ignore -->
            <!-- svelte-ignore binding_property_non_reactive -->
            <TaskRow bind:this={rows[task.id]} {index} {task} {users} />
          </tbody>
        {/each}
      </table>
    {:else}
      <div class="flex items-center justify-center pt-8">
        <div
          class="bg-m3-surface-container flex w-9/12 max-w-[425px] rounded-md border p-4"
        >
          <div class="min-w-16">
            <KosoLogo class="size-16" />
          </div>
          <div class="ml-4">
            <div class="text-md">Inbox zero!</div>
            <div class="mt-2 text-sm">
              You've achieved inbox zero. Great job!
            </div>
          </div>
        </div>
      </div>
    {/if}
  {/await}
</Toolbar>
