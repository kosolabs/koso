<script lang="ts">
  import { replaceState } from "$app/navigation";
  import { auth } from "$lib/auth.svelte";
  import {
    getRegistryContext,
    type ActionID,
  } from "$lib/components/ui/command-palette";
  import { KosoLogo } from "$lib/components/ui/koso-logo";
  import { toast } from "$lib/components/ui/sonner";
  import { Button } from "$lib/kosui/button";
  import { Action } from "$lib/kosui/command";
  import { Fab } from "$lib/kosui/fab";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { CANCEL, INSERT_CHILD_NODE, INSERT_NODE } from "$lib/shortcuts";
  import type { User } from "$lib/users";
  import {
    Cable,
    Check,
    ChevronsDownUp,
    ChevronsUpDown,
    CircleX,
    Clipboard,
    Eye,
    EyeOff,
    IndentDecrease,
    IndentIncrease,
    ListEnd,
    ListPlus,
    ListStart,
    ListTree,
    MoveDown,
    MoveUp,
    OctagonX,
    Pencil,
    Plus,
    Redo,
    Search,
    SkipBack,
    SkipForward,
    SquarePen,
    StepBack,
    StepForward,
    Trash,
    Undo,
    UserRoundPlus,
    Wrench,
  } from "lucide-svelte";
  import { onMount, tick } from "svelte";
  import { flip } from "svelte/animate";
  import DagRow from "./dag-row.svelte";
  import { getPlanningContext, Node } from "./planning-context.svelte";
  import SearchPanel from "./search-panel.svelte";

  type Props = {
    users: User[];
  };
  const { users }: Props = $props();

  const rows: { [key: string]: DagRow } = {};

  const command = getRegistryContext();
  const planningCtx = getPlanningContext();
  const { koso } = planningCtx;

  function getRow(node: Node) {
    const maybeRow = rows[node.id];
    if (!maybeRow) {
      throw new Error(`Row doesn't exist for ${node}`);
    }
    return maybeRow;
  }

  let searchPaletteOpen: boolean = $state(false);
  function showSearchPalette() {
    searchPaletteOpen = true;
  }

  function insertAndEdit(parent: Node, offset: number, user: User) {
    const taskId = koso.insertTask(parent.name, offset, user);
    const node = parent.child(taskId);
    planningCtx.selected = node;

    // The newly inserted node's row won't yet have been inserted into
    // the dom and thus onMount will not have been called to register
    // row callbacks.
    // Delay interacting with the row registry to start editing.
    tick().then(() => getRow(node).edit(true));
  }

  function insert() {
    if (planningCtx.selected) {
      insertAndEdit(
        planningCtx.selected.parent,
        planningCtx.getOffset(planningCtx.selected) + 1,
        auth.user,
      );
    } else {
      insertAndEdit(planningCtx.root, 0, auth.user);
    }
  }

  function insertAbove() {
    if (!planningCtx.selected) return;
    insertAndEdit(
      planningCtx.selected.parent,
      planningCtx.getOffset(planningCtx.selected),
      auth.user,
    );
  }

  function insertChild() {
    if (!planningCtx.selected) return;
    planningCtx.expand(planningCtx.selected);
    insertAndEdit(planningCtx.selected, 0, auth.user);
  }

  function insertChildAbove() {
    if (!planningCtx.selected) return;

    const previousPeer = planningCtx.getPrevPeer(planningCtx.selected);
    if (!previousPeer) return;

    planningCtx.expand(previousPeer);
    const lastIndex = koso.getChildCount(previousPeer.name);
    insertAndEdit(previousPeer, lastIndex, auth.user);
  }

  function toggleStatus() {
    if (!planningCtx.selected) return;

    const task = koso.getTask(planningCtx.selected.name);
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
        koso.setTaskStatus(planningCtx.selected.name, "Not Started", auth.user);
        return;
      case "In Progress": {
        const node = planningCtx.selected;

        getRow(node).showDoneConfetti();
        koso.setTaskStatus(node.name, "Done", auth.user);
        break;
      }
      case "Not Started":
        koso.setTaskStatus(planningCtx.selected.name, "In Progress", auth.user);
        break;
      default:
        throw new Error(`Unhandled status ${task.yStatus}`);
    }
  }

  function remove() {
    if (!planningCtx.selected) return;
    const toDelete = planningCtx.selected;
    const toDeleteIndex = planningCtx.nodes.indexOf(toDelete);

    koso.delete(toDelete.linkage);

    // Select the next (or previous) node following deletion.
    if (planningCtx.nodes.size < 2 || toDeleteIndex <= 0) {
      planningCtx.selected = null;
    } else {
      planningCtx.selected =
        planningCtx.nodes.get(
          Math.min(toDeleteIndex, planningCtx.nodes.size - 1),
        ) || null;
    }
  }

  function edit() {
    if (!planningCtx.selected) return;
    getRow(planningCtx.selected).edit(true);
  }

  function unselect() {
    planningCtx.selected = null;
  }

  function moveUp() {
    if (!planningCtx.selected) return;
    planningCtx.moveNodeUp(planningCtx.selected);
  }

  function moveDown() {
    if (!planningCtx.selected) return;
    planningCtx.moveNodeDown(planningCtx.selected);
  }

  function moveStart() {
    if (!planningCtx.selected) return;
    planningCtx.moveNodeUpBoundary(planningCtx.selected);
  }

  function moveEnd() {
    if (!planningCtx.selected) return;
    planningCtx.moveNodeDownBoundary(planningCtx.selected);
  }

  function indent() {
    if (!planningCtx.selected) return;
    planningCtx.indentNode(planningCtx.selected);
  }

  function undent() {
    if (!planningCtx.selected) return;
    planningCtx.undentNode(planningCtx.selected);
  }

  function expand() {
    if (!planningCtx.selected) return;
    planningCtx.expand(planningCtx.selected);
  }

  function collapse() {
    if (!planningCtx.selected) return;
    planningCtx.collapse(planningCtx.selected);
  }

  function showDoneTasks() {
    planningCtx.showDone = true;
  }

  function hideDoneTasks() {
    planningCtx.showDone = false;
  }

  function selectNext() {
    if (planningCtx.nodes.size > 1) {
      if (planningCtx.selected) {
        const selectedIndex = planningCtx.nodes.indexOf(planningCtx.selected);
        if (selectedIndex <= 0) {
          planningCtx.selected = null;
        } else {
          const index = Math.min(selectedIndex + 1, planningCtx.nodes.size - 1);
          planningCtx.selected = planningCtx.nodes.get(index, null);
        }
      } else {
        planningCtx.selected = planningCtx.nodes.get(1, null);
      }
    }
  }

  function selectPrev() {
    if (planningCtx.nodes.size > 1) {
      if (planningCtx.selected) {
        const selectedIndex = planningCtx.nodes.indexOf(planningCtx.selected);
        if (selectedIndex <= 0) {
          planningCtx.selected = null;
        } else {
          const index = Math.max(selectedIndex - 1, 1);
          planningCtx.selected = planningCtx.nodes.get(index, null);
        }
      } else {
        planningCtx.selected = planningCtx.nodes.get(
          planningCtx.nodes.size - 1,
          null,
        );
      }
    }
  }

  function selectNextLink() {
    if (planningCtx.selected) {
      const next = planningCtx.getNextLink(planningCtx.selected);
      if (next) {
        planningCtx.selected = next;
      }
    }
  }

  function selectPrevLink() {
    if (planningCtx.selected) {
      const prev = planningCtx.getPrevLink(planningCtx.selected);
      if (prev) {
        planningCtx.selected = prev;
      }
    }
  }

  function undo() {
    planningCtx.undo();
  }

  function redo() {
    planningCtx.redo();
  }

  function linkTask() {
    if (!planningCtx.selected) return;
    getRow(planningCtx.selected).linkPanel(true, "link");
  }

  function blockTask() {
    if (!planningCtx.selected) return;
    getRow(planningCtx.selected).linkPanel(true, "block");
  }

  function organizeTasks() {
    if (!planningCtx.selected) return;
    koso.organizeTasks(planningCtx.selected.parent.name);
  }

  function copyGitCommitMessage() {
    if (!planningCtx.selected) return;
    const taskId = planningCtx.selected.name;
    navigator.clipboard.writeText(koso.getGitCommitMessage(taskId));
  }

  const insertAction: Action<ActionID> = {
    id: "Insert",
    callback: insert,
    title: "Add Task",
    description: "Add or insert a new task",
    icon: ListPlus,
    shortcut: INSERT_NODE,
    enabled: () =>
      !planningCtx.selected || koso.canInsert(planningCtx.selected.parent.name),
  };

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
      id: "Expand",
      callback: expand,
      description: "Expand the current task",
      icon: ChevronsUpDown,
      enabled: () =>
        !!planningCtx.selected && planningCtx.canExpand(planningCtx.selected),
      shortcut: new Shortcut({ key: "ArrowRight" }),
    }),
    new Action({
      id: "Collapse",
      callback: collapse,
      description: "Collapse the current task",
      icon: ChevronsDownUp,
      enabled: () =>
        !!planningCtx.selected && planningCtx.canCollapse(planningCtx.selected),
      shortcut: new Shortcut({ key: "ArrowLeft" }),
    }),
    new Action({
      id: "ExpandAll",
      callback: () => planningCtx.expandAll(),
      title: "Expand All",
      description: "Expand all tasks",
      icon: ChevronsUpDown,
    }),
    new Action({
      id: "CollapseAll",
      callback: () => planningCtx.collapseAll(),
      title: "Collapse All",
      description: "Collapse all tasks",
      icon: ChevronsDownUp,
    }),
    insertAction,
    new Action({
      id: "InsertAbove",
      callback: insertAbove,
      title: "Insert Above",
      description: "Insert a new task above",
      icon: ListPlus,
      shortcut: new Shortcut({ key: "Enter", meta: true, shift: true }),
      enabled: () =>
        !!planningCtx.selected &&
        koso.canInsert(planningCtx.selected.parent.name),
    }),
    new Action({
      id: "InsertSubtask",
      callback: insertChild,
      title: "Insert subtask",
      description: "Insert a new task as a child",
      icon: ListTree,
      enabled: () =>
        !!planningCtx.selected && koso.canInsert(planningCtx.selected.name),
      shortcut: INSERT_CHILD_NODE,
    }),
    new Action({
      id: "InsertSubtaskAbove",
      callback: insertChildAbove,
      title: "Insert subtask above",
      description: "Insert a new task as a child of the previous task",
      icon: ListTree,
      enabled: () => {
        if (!planningCtx.selected) {
          return false;
        }
        const prevPeer = planningCtx.getPrevPeer(planningCtx.selected);
        return !!prevPeer && koso.canInsert(prevPeer.name);
      },
      shortcut: new Shortcut({
        key: "Enter",
        alt: true,
        meta: true,
        shift: true,
      }),
    }),
    new Action({
      id: "Edit",
      callback: edit,
      description: "Edit the current task",
      icon: Pencil,
      shortcut: new Shortcut({ key: "Enter" }),
      enabled: () =>
        !!planningCtx.selected && koso.isEditable(planningCtx.selected.name),
    }),
    new Action({
      id: "Clear",
      callback: unselect,
      description: "Clear the current selection",
      icon: CircleX,
      shortcut: CANCEL,
    }),
    new Action({
      id: "Delete",
      callback: remove,
      title: "Delete task",
      description: "Delete the current task",
      icon: Trash,
      enabled: () =>
        !!planningCtx.selected && koso.canDelete(planningCtx.selected.linkage),
      shortcut: new Shortcut({ key: "Delete" }),
    }),
    new Action({
      id: "MoveUp",
      callback: moveUp,
      title: "Move up",
      description: "Move the current task up",
      icon: MoveUp,
      enabled: () => !!planningCtx.selected,
      shortcut: new Shortcut({ key: "ArrowUp", alt: true }),
    }),
    new Action({
      id: "MoveDown",
      callback: moveDown,
      title: "Move down",
      description: "Move the current task down",
      icon: MoveDown,
      enabled: () => !!planningCtx.selected,
      shortcut: new Shortcut({ key: "ArrowDown", alt: true }),
    }),
    new Action({
      id: "MoveToStart",
      callback: moveStart,
      title: "Move to start",
      description: "Move the current task to the top of its group",
      icon: ListStart,
      enabled: () => !!planningCtx.selected,
      shortcut: new Shortcut({ key: "ArrowUp", alt: true, shift: true }),
    }),
    new Action({
      id: "MoveToEnd",
      callback: moveEnd,
      title: "Move to end",
      description: "Move the current task to the bottom of its group",
      icon: ListEnd,
      enabled: () => !!planningCtx.selected,
      shortcut: new Shortcut({ key: "ArrowDown", alt: true, shift: true }),
    }),
    new Action({
      id: "Undent",
      callback: undent,
      title: "Unindent task",
      description: "Make the current task a peer of its parent",
      icon: IndentDecrease,
      enabled: () =>
        !!planningCtx.selected &&
        planningCtx.canUndentNode(planningCtx.selected),
      shortcut: new Shortcut({ key: "ArrowLeft", alt: true }),
    }),
    new Action({
      id: "Indent",
      callback: indent,
      title: "Indent task",
      description: "Make the current task a child of its peer",
      icon: IndentIncrease,
      enabled: () =>
        !!planningCtx.selected &&
        planningCtx.canIndentNode(planningCtx.selected),
      shortcut: new Shortcut({ key: "ArrowRight", alt: true }),
    }),
    new Action({
      id: "Undo",
      callback: undo,
      icon: Undo,
      shortcut: new Shortcut({ key: "z", meta: true }),
    }),
    new Action({
      id: "Redo",
      callback: redo,
      icon: Redo,
      shortcut: new Shortcut({ key: "z", meta: true, shift: true }),
    }),
    new Action({
      id: "ToggleTaskStatus",
      callback: toggleStatus,
      title: "Toggle Task Status",
      description: "Toggle the task status to In Progress or Done",
      icon: Check,
      shortcut: new Shortcut({ key: " " }),
      enabled: () =>
        !!planningCtx.selected && koso.isEditable(planningCtx.selected.name),
    }),
    new Action({
      id: "HideDoneTasks",
      callback: hideDoneTasks,
      title: "Hide Done Tasks",
      description: "Hide tasks that have been marked done",
      icon: EyeOff,
      enabled: () => planningCtx.showDone,
    }),
    new Action({
      id: "ShowDoneTasks",
      callback: showDoneTasks,
      title: "Show Done Tasks",
      description: "Show tasks that have been marked done",
      icon: Eye,
      enabled: () => !planningCtx.showDone,
    }),
    new Action({
      id: "Search",
      callback: showSearchPalette,
      description: "Show the search palette",
      icon: Search,
      shortcut: new Shortcut({ key: "p", meta: true }),
    }),
    new Action({
      id: "NextLink",
      callback: selectNextLink,
      title: "Next Link",
      description: "Select next link to current task",
      icon: SkipForward,
      enabled: () => !!planningCtx.selected,
      shortcut: new Shortcut({ key: "ArrowDown", meta: true }),
    }),
    new Action({
      id: "PreviousLink",
      callback: selectPrevLink,
      title: "Previous Link",
      description: "Select previous link to current task",
      icon: SkipBack,
      enabled: () => !!planningCtx.selected,
      shortcut: new Shortcut({ key: "ArrowUp", meta: true }),
    }),
    new Action({
      id: "Link",
      callback: linkTask,
      title: "Link task to...",
      description: "Link current task to another task",
      icon: Cable,
      enabled: () => !!planningCtx.selected,
      shortcut: new Shortcut({ key: "/", meta: true }),
    }),
    new Action({
      id: "Block",
      callback: blockTask,
      title: "Block task on...",
      description: "Block current task to another task",
      icon: OctagonX,
      enabled: () => !!planningCtx.selected,
    }),
    new Action({
      id: "Organize",
      callback: organizeTasks,
      title: "Organize Tasks",
      description: "Organize the current task and its peers",
      icon: Wrench,
      enabled: () => !!planningCtx.selected,
    }),
    new Action({
      id: "CopyTaskInfo",
      callback: copyGitCommitMessage,
      title: "Copy task info",
      description: "Copy task git commit message to the clipboard",
      icon: Clipboard,
      shortcut: new Shortcut({ key: "c", meta: true }),
      enabled: () => !!planningCtx.selected,
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
      if (planningCtx.koso.getTaskIndex(taskId) < 0) {
        console.debug(
          `Waiting for server sync before selecting task ${taskId}`,
        );
        await koso.serverSynced;
        await tick();

        if (planningCtx.koso.getTaskIndex(taskId) < 0) {
          console.warn(
            `Cannot select ${taskId} after server sync. It doesn't exist`,
          );
          toast.warning(`Task not found. It may have been deleted.`);
          return;
        }
      }

      planningCtx.select(taskId);
    }
  });

  onMount(() => {
    // Clear selection on destroy to avoid persisting awareness on navigation.
    return () => {
      planningCtx.selected = null;
    };
  });

  onMount(() => {
    return command.register(...actions);
  });

  // This effect selects a new node when the
  // selected node no longer exists. For example, when
  // the user marks a task as done or blocked in the inbox
  // or a different user deletes the user's currently selected node.
  $effect(() => {
    const selected = planningCtx.selectedRaw;
    const node = selected.node;
    const index = selected.index;

    if (!node) {
      return;
    }

    const currentIndex = planningCtx.nodes.indexOf(node);
    if (currentIndex !== -1) {
      // The node still exists. Make sure the stashed index still matches.
      if (!index || index !== currentIndex) {
        console.debug(
          `Refreshing selected index for node ${node.id} at prior index ${index}`,
        );
        planningCtx.selected = node;
      }
      return;
    }

    // The selected node no longer exists. Select the
    // node at the same index or the one at the end of the list.
    // The first node is not selectable.
    if (planningCtx.nodes.size > 1) {
      console.debug(`Node ${node.id} no longer exists. Selecting new node.`);
      planningCtx.selected = planningCtx.nodes.get(
        Math.min(index || -1, planningCtx.nodes.size - 1),
        null,
      );
    } else {
      console.debug(`Node ${node.id} no longer exists. Clearing selection.`);
      planningCtx.selected = null;
    }
  });
</script>

<SearchPanel bind:open={searchPaletteOpen} />

{#await koso.synced then}
  {#if planningCtx.nodes.size > 1}
    <div class="relative h-full">
      <div class="flex flex-col gap-2">
        <!-- Add a z-0 to fix a bug in Safari where rows disappear when collapsing
                 and expanding tasks -->
        <table
          class="dag-table z-0 w-full border-separate border-spacing-0 rounded-md border"
        >
          <thead class="text-left text-xs font-bold uppercase">
            <tr>
              <th class="relative m-0 w-0 p-0"></th>
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
              <th class="border-l p-2 max-md:hidden">Reporter</th>
              <th class="relative m-0 w-0 p-0"></th>
            </tr>
          </thead>

          {#each [...planningCtx.nodes].slice(1) as node, index (node.id)}
            <tbody animate:flip={{ duration: 250 }}>
              <!-- eslint-disable-next-line svelte/no-unused-svelte-ignore -->
              <!-- svelte-ignore binding_property_non_reactive -->
              <DagRow bind:this={rows[node.id]} {index} {node} {users} />
            </tbody>
          {/each}
        </table>

        <Fab
          reserveClass="m-0.5"
          positionClass="m-0"
          icon={Plus}
          onclick={insertAction.callback}
          reserve
        >
          {insertAction.title}
          {#snippet tooltip()}
            <div class="flex items-center gap-2">
              {insertAction.description}
              {#if insertAction.shortcut}
                <div class="font-bold">
                  {insertAction.shortcut.toString()}
                </div>
              {/if}
            </div>
          {/snippet}
        </Fab>
      </div>
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
          <div class="text-md">Welcome to Koso!</div>
          <div class="mt-2 text-sm">
            Koso helps you to organize your work and be productive.
          </div>
          <div class="mt-4">
            <Button variant="filled" icon={ListPlus} onclick={insert}>
              Add task
            </Button>
          </div>
        </div>
      </div>
    </div>
  {/if}
{/await}

<!-- Round the bottom left and right of the table -->
<style>
  :global(.dag-table > tbody:last-child > tr > td:nth-child(2)) {
    border-bottom-left-radius: 0.25rem;
  }

  :global(.dag-table > tbody:last-child > tr > td:nth-last-child(2)) {
    border-bottom-right-radius: 0.25rem;
  }
</style>
