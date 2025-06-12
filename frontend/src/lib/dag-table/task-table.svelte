<script lang="ts">
  import { goto, replaceState } from "$app/navigation";
  import { getAuthContext } from "$lib/auth.svelte";
  import { getRegistryContext } from "$lib/components/ui/command-palette";
  import {
    ActionIds,
    Categories,
  } from "$lib/components/ui/command-palette/command-palette.svelte";
  import KosoLogo from "$lib/components/ui/koso-logo/koso-logo.svelte";
  import { getPrefsContext } from "$lib/components/ui/prefs";
  import { toast } from "$lib/components/ui/sonner";
  import { Action } from "$lib/kosui/command";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { CANCEL } from "$lib/shortcuts";
  import type { User } from "$lib/users";
  import {
    Cable,
    Check,
    CircleGauge,
    CircleX,
    Clipboard,
    Hash,
    OctagonX,
    Pencil,
    Redo,
    Share,
    SquarePen,
    StepBack,
    StepForward,
    Trash,
    Undo,
    UserRoundPlus,
  } from "lucide-svelte";
  import { onMount, tick } from "svelte";
  import { flip } from "svelte/animate";
  import { getInboxContext } from "./inbox-context.svelte";
  import { getProjectContext } from "./project-context.svelte";
  import TaskEstimateHeading from "./task-estimate-heading.svelte";
  import TaskRow from "./task-row.svelte";

  type Props = {
    users: User[];
  };
  const { users }: Props = $props();

  const prefs = getPrefsContext();
  const auth = getAuthContext();
  const projectCtx = getProjectContext();
  const command = getRegistryContext();
  const inbox = getInboxContext();
  const { koso } = inbox;

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
    koso.toggleStatus(inbox.selected.id, auth.user);
  }

  function edit() {
    if (!inbox.selected) return;
    getRow(inbox.selected.id).edit(true);
  }

  function remove() {
    if (!inbox.selected) return;
    koso.deleteTask(inbox.selected.id);
  }

  function unselect() {
    inbox.selected = undefined;
  }

  function selectNext() {
    if (inbox.actionItems.length > 0) {
      if (inbox.selected) {
        const selectedIndex = inbox.getTaskIndex(inbox.selected.id);
        if (selectedIndex < 0) {
          inbox.selected = undefined;
        } else {
          const index = Math.min(
            selectedIndex + 1,
            inbox.actionItems.length - 1,
          );
          inbox.selected = inbox.actionItems[index].task.id;
        }
      } else {
        inbox.selected = inbox.actionItems[0].task.id;
      }
    }
  }

  function selectPrev() {
    if (inbox.actionItems.length > 0) {
      if (inbox.selected) {
        const selectedIndex = inbox.getTaskIndex(inbox.selected.id);
        if (selectedIndex < 0) {
          inbox.selected = undefined;
        } else {
          const index = Math.max(selectedIndex - 1, 0);
          inbox.selected = inbox.actionItems[index].task.id;
        }
      } else {
        inbox.selected =
          inbox.actionItems[inbox.actionItems.length - 1].task.id;
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

  function copyTaskId() {
    if (!inbox.selected) return;
    const taskId = inbox.selected.id;
    navigator.clipboard.writeText(koso.getGitCommitId(taskId));
  }

  function copyTaskLink() {
    if (!inbox.selected) return;
    navigator.clipboard.writeText(
      koso.getTaskPermalink(inbox.selected.id).toString(),
    );
  }

  const actions: Action[] = [
    new Action({
      id: ActionIds.Next,
      callback: selectNext,
      category: Categories.Select,
      name: "Next Task",
      description: "Select next task",
      icon: StepForward,
      shortcut: new Shortcut({ key: "ArrowDown" }),
    }),
    new Action({
      id: ActionIds.Previous,
      callback: selectPrev,
      category: Categories.Select,
      name: "Previous Task",
      description: "Select previous task",
      icon: StepBack,
      shortcut: new Shortcut({ key: "ArrowUp" }),
    }),
    new Action({
      id: ActionIds.Clear,
      callback: unselect,
      category: Categories.Select,
      name: "Deselect Task",
      description: "Clear the current selection",
      icon: CircleX,
      shortcut: CANCEL,
    }),

    new Action({
      id: ActionIds.Undo,
      callback: undo,
      category: Categories.Edit,
      name: "Undo",
      icon: Undo,
      shortcut: new Shortcut({ key: "z", meta: true }),
    }),
    new Action({
      id: ActionIds.Redo,
      callback: redo,
      category: Categories.Edit,
      name: "Redo",
      icon: Redo,
      shortcut: new Shortcut({ key: "z", meta: true, shift: true }),
    }),
    new Action({
      id: ActionIds.Edit,
      callback: edit,
      category: Categories.Edit,
      name: "Edit Task Name",
      description: "Edit the current task",
      icon: Pencil,
      shortcut: new Shortcut({ key: "Enter" }),
      enabled: () => !!inbox.selected && koso.isEditable(inbox.selected.id),
    }),
    new Action({
      id: ActionIds.ToggleTaskStatus,
      callback: toggleStatus,
      category: Categories.Edit,
      name: "Toggle Task Status",
      description: "Toggle the task status to In Progress or Done",
      icon: Check,
      shortcut: new Shortcut({ key: " " }),
      enabled: () => !!inbox.selected && koso.isEditable(inbox.selected.id),
    }),
    new Action({
      id: ActionIds.Delete,
      callback: remove,
      category: Categories.Edit,
      name: "Delete Task",
      description: "Delete the current task",
      icon: Trash,
      enabled: () =>
        !!inbox.selected &&
        koso.isEditable(inbox.selected.id) &&
        koso.canDeleteTask(inbox.selected.id),
      shortcut: new Shortcut({ key: "Delete" }),
    }),
    new Action({
      id: ActionIds.CopyTaskInfo,
      callback: copyTaskId,
      category: Categories.Edit,
      name: "Copy Task ID",
      description: "Copy task ID to the clipboard",
      icon: Clipboard,
      enabled: () => !!inbox.selected,
    }),
    new Action({
      id: ActionIds.CopyTaskLink,
      callback: copyTaskLink,
      category: Categories.Edit,
      name: "Copy Task Permalink",
      description: "Share task by copying permalink to the clipboard",
      icon: Share,
      shortcut: new Shortcut({ key: "c", meta: true, shift: true }),
      enabled: () => !!inbox.selected,
    }),

    new Action({
      id: ActionIds.Link,
      callback: linkTask,
      category: Categories.Graph,
      name: "Link Task To...",
      description: "Link current task to another task",
      icon: Cable,
      enabled: () => !!inbox.selected,
    }),
    new Action({
      id: ActionIds.Block,
      callback: blockTask,
      category: Categories.Graph,
      name: "Block Task On...",
      description: "Block current task to another task",
      icon: OctagonX,
      enabled: () => !!inbox.selected,
      shortcut: new Shortcut({ key: "/", meta: true }),
    }),

    new Action({
      id: ActionIds.DashView,
      callback: () =>
        goto(`/projects/${projectCtx.id}/dash/${inbox.selected?.id}`),
      category: Categories.Navigation,
      name: "Dashboard",
      description: "Navigate to Project Dashboard view",
      icon: CircleGauge,
      enabled: () =>
        !!inbox.selected &&
        inbox.selected.isRollup() &&
        inbox.selected.deadline !== null,
    }),
  ];

  onMount(async () => {
    const url = new URL(window.location.href);
    const taskId = url.searchParams.get("taskId");
    if (taskId) {
      await koso.synced;
      url.searchParams.delete("taskId");
      replaceState(url, {});

      // The task may not exist locally, yet. It
      // might come from the server, so wait for that.
      if (inbox.getTaskIndex(taskId) < 0) {
        console.debug(
          `Waiting for server sync before selecting task ${taskId}`,
        );
        await koso.serverSynced;
        await tick();

        if (inbox.getTaskIndex(taskId) < 0) {
          console.warn(
            `Cannot select ${taskId} after server sync. It doesn't exist`,
          );
          toast.warning(
            `Task not found. It may have been removed from your inbox.`,
          );
          return;
        }
      }
      inbox.selected = taskId;
    }
  });

  onMount(() => {
    return command.register(...actions);
  });

  // This effect selects a new task when the
  // selected task no longer exists. For example, when
  // the user marks a task as done or blocked in the inbox
  // or a different user deletes the user's currently selected task.
  $effect(() => {
    const selected = inbox.selectedRaw;
    const taskId = selected.taskId;
    const index = selected.index;

    if (!taskId) {
      return;
    }

    const currentIndex = inbox.getTaskIndex(taskId);
    if (currentIndex !== -1) {
      // The task still exists. Make sure the stashed index still matches.
      if (index === null || index !== currentIndex) {
        console.debug(
          `Refreshing selected index for task ${taskId} at prior index ${index}, currentIndex ${currentIndex}`,
        );
        inbox.selected = taskId;
      }
      return;
    }

    // The selected task no longer exists. Select the
    // task at the same index or the one at the end of the list.
    if (inbox.actionItems.length > 0) {
      console.debug(`Task ${taskId} no longer exists. Selecting new task.`);
      if (index === null || index >= inbox.actionItems.length) {
        inbox.selected =
          inbox.actionItems[inbox.actionItems.length - 1].task.id;
      } else {
        inbox.selected = inbox.actionItems[index].task.id;
      }
    } else {
      console.debug(`Task ${taskId} no longer exists. Clearing selection.`);
      inbox.selected = undefined;
    }
  });
</script>

{#await koso.synced then}
  {#if inbox.actionItems.length > 0}
    <div class="flex flex-col gap-2">
      <table
        class="task-table w-full border-separate border-spacing-0 rounded-md border"
      >
        <thead class="text-left text-xs font-bold uppercase">
          <tr>
            <th class="p-2">
              <div class="flex items-center" title="ID">
                <Hash class="h-4" />
                <div class="max-md:hidden">ID</div>
              </div>
            </th>
            {#if prefs.debug}
              <th class="border-l p-2">UUID</th>
            {/if}
            <th class="border-l p-2">
              <div class="flex items-center" title="Status">
                <SquarePen class="h-4" />
                <div class="max-md:hidden">Status</div>
              </div>
            </th>
            <th class="border-l p-2">Name</th>
            <th class="p-2"></th>
            <th class="border-l p-2">
              <div class="flex items-center" title="Assignee">
                <UserRoundPlus class="h-4" />
                <div class="max-md:hidden">Assignee</div>
              </div>
            </th>
            <th class="border-l max-md:hidden">
              <TaskEstimateHeading />
            </th>
            <th class="relative m-0 w-0 p-0"></th>
          </tr>
        </thead>

        {#each inbox.actionItems as actionItem, index (actionItem.task.id)}
          <tbody animate:flip={{ duration: 250 }}>
            <!-- eslint-disable-next-line svelte/no-unused-svelte-ignore -->
            <!-- svelte-ignore binding_property_non_reactive -->
            <TaskRow
              bind:this={rows[actionItem.task.id]}
              {index}
              item={actionItem}
              {users}
            />
          </tbody>
        {/each}
      </table>
    </div>
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
          <div class="mt-2 text-sm">You've achieved inbox zero. Great job!</div>
        </div>
      </div>
    </div>
  {/if}
{/await}

<!-- Round the bottom left and right of the table -->
<style>
  :global(.task-table > tbody:last-child > tr > td:first-child) {
    border-bottom-left-radius: 0.25rem;
  }

  :global(.task-table > tbody:last-child > tr > td:last-child) {
    border-bottom-right-radius: 0.25rem;
  }
</style>
