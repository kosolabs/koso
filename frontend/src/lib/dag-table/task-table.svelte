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
  import { Node, type Koso } from ".";
  import MarkdownEditor from "./markdown-editor.svelte";
  import TaskRow from "./task-row.svelte";
  import Toolbar from "./toolbar.svelte";

  type Props = {
    koso: Koso;
    users: User[];
  };
  const { koso, users }: Props = $props();

  const rows: { [key: string]: TaskRow } = {};

  function getRow(node: Node) {
    const maybeRow = rows[node.id];
    if (!maybeRow) {
      throw new Error(`Row doesn't exist for ${node}`);
    }
    return maybeRow;
  }

  function toggleStatus() {
    if (!koso.selected) return;

    const task = koso.getTask(koso.selected.name);
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
    if (!koso.selected) return;
    getRow(koso.selected).edit(true);
  }

  function unselect() {
    koso.selected = null;
  }

  function selectNext() {
    if (koso.nodes.size > 1) {
      if (koso.selected) {
        const selectedIndex = koso.nodes.indexOf(koso.selected);
        if (selectedIndex <= 0) {
          koso.selected = null;
        } else {
          const index = Math.min(selectedIndex + 1, koso.nodes.size - 1);
          koso.selected = koso.nodes.get(index, null);
        }
      } else {
        koso.selected = koso.nodes.get(1, null);
      }
    }
  }

  function selectPrev() {
    if (koso.nodes.size > 1) {
      if (koso.selected) {
        const selectedIndex = koso.nodes.indexOf(koso.selected);
        if (selectedIndex <= 0) {
          koso.selected = null;
        } else {
          const index = Math.max(selectedIndex - 1, 1);
          koso.selected = koso.nodes.get(index, null);
        }
      } else {
        koso.selected = koso.nodes.get(koso.nodes.size - 1, null);
      }
    }
  }

  function undo() {
    koso.undo();
  }

  function redo() {
    koso.redo();
  }

  function linkTask() {
    if (!koso.selected) return;
    getRow(koso.selected).linkPanel(true, "link");
  }

  function blockTask() {
    if (!koso.selected) return;
    getRow(koso.selected).linkPanel(true, "block");
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
      enabled: () => !!koso.selected && koso.isEditable(koso.selected.name),
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
      enabled: () => !!koso.selected && koso.isEditable(koso.selected.name),
    }),
    new Action({
      id: "Link",
      callback: linkTask,
      title: "Link task to...",
      description: "Link current task to another task",
      icon: Cable,
      enabled: () => !!koso.selected,
    }),
    new Action({
      id: "Block",
      callback: blockTask,
      title: "Block task on...",
      description: "Block current task to another task",
      icon: OctagonX,
      enabled: () => !!koso.selected,
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
      koso.select(taskId);
    }
  });

  setContext<Koso>("koso", koso);

  onMount(() => {
    return command.register(...actions);
  });

  // This effect selects a new node when the
  // selected node no longer exists. For example, when
  // the user marks a task as done or blocked in the inbox
  // or a different user deletes the user's currently selected node.
  $effect(() => {
    const selected = koso.selectedRaw;
    const node = selected.node;
    const index = selected.index;

    if (!node) {
      return;
    }

    const currentIndex = koso.nodes.indexOf(node);
    if (currentIndex !== -1) {
      // The node still exists. Make sure the stashed index still matches.
      if (!index || index !== currentIndex) {
        console.debug(
          `Refreshing selected index for node ${node.id} at prior index ${index}`,
        );
        koso.selected = node;
      }
      return;
    }

    // The selected node no longer exists. Select the
    // node at the same index or the one at the end of the list.
    // The first node is not selectable.
    if (koso.nodes.size > 1) {
      console.debug(`Node ${node.id} no longer exists. Selecting new node.`);
      koso.selected = koso.nodes.get(
        Math.min(index || -1, koso.nodes.size - 1),
        null,
      );
    } else {
      console.debug(`Node ${node.id} no longer exists. Clearing selection.`);
      koso.selected = null;
    }
  });
</script>

<Toolbar actions={[undoAction, redoAction]}>
  {#await koso.synced then}
    {#if koso.nodes.size > 1}
      {#if koso.selected}
        <MarkdownEditor {koso} task={koso.getTask(koso.selected.name)} />
      {/if}

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

        {#each [...koso.tasks] as task, index (task.id)}
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
