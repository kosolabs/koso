<script lang="ts">
  import { replaceState } from "$app/navigation";
  import { auth, type User } from "$lib/auth.svelte";
  import { command, type ActionID } from "$lib/components/ui/command-palette";
  import KosoLogo from "$lib/components/ui/koso-logo/koso-logo.svelte";
  import { toast } from "$lib/components/ui/sonner";
  import { Button } from "$lib/kosui/button";
  import { Action } from "$lib/kosui/command";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { CANCEL, INSERT_CHILD_NODE, INSERT_NODE } from "$lib/shortcuts";
  import {
    Cable,
    Check,
    ChevronsDownUp,
    ChevronsUpDown,
    CircleX,
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
  import { onMount, setContext, tick } from "svelte";
  import { flip } from "svelte/animate";
  import { Node, type Koso } from ".";
  import Row from "./row.svelte";
  import SearchPanel from "./search-panel.svelte";
  import Toolbar from "./toolbar.svelte";

  type Props = {
    koso: Koso;
    users: User[];
    inboxView: boolean;
  };
  const { koso, users, inboxView }: Props = $props();

  const rows: { [key: string]: Row } = {};

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
    const node = koso.insertNode(parent, offset, user);
    // The newly inserted node's row won't yet have been inserted into
    // the dom and thus onMount will not have been called to register
    // row callbacks.
    // Delay interacting with the row registry to start editing.
    tick().then(() => getRow(node).edit(true));
  }

  function insert() {
    if (koso.selected) {
      insertAndEdit(
        koso.selected.parent,
        koso.getOffset(koso.selected) + 1,
        auth.user,
      );
    } else {
      insertAndEdit(koso.root, 0, auth.user);
    }
  }

  function insertAbove() {
    if (!koso.selected) return;
    insertAndEdit(
      koso.selected.parent,
      koso.getOffset(koso.selected),
      auth.user,
    );
  }

  function insertChild() {
    if (!koso.selected) return;
    koso.expand(koso.selected);
    insertAndEdit(koso.selected, 0, auth.user);
  }

  function insertChildAbove() {
    if (!koso.selected) return;

    const previousPeer = koso.getPrevPeer(koso.selected);
    if (!previousPeer) return;

    koso.expand(previousPeer);
    const lastIndex = koso.getChildCount(previousPeer.name);
    insertAndEdit(previousPeer, lastIndex, auth.user);
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
        koso.setTaskStatus(koso.selected, "Not Started", auth.user);
        return;
      case "In Progress": {
        const node = koso.selected;

        getRow(node).showDoneConfetti();
        koso.setTaskStatus(node, "Done", auth.user);
        if (inboxView) {
          toast.success("🚀 Great work! Task complete!");
        }
        break;
      }
      case "Not Started":
        koso.setTaskStatus(koso.selected, "In Progress", auth.user);
        break;
      default:
        throw new Error(`Unhandled status ${task.yStatus}`);
    }
  }

  function remove() {
    if (!koso.selected) return;
    const toDelete = koso.selected;
    const toDeleteIndex = koso.nodes.indexOf(toDelete);

    koso.deleteNode(toDelete);

    if (!inboxView) {
      // Select the next (or previous) node following deletion.
      if (koso.nodes.size < 2 || toDeleteIndex <= 0) {
        koso.selected = null;
      } else {
        koso.selected =
          koso.nodes.get(Math.min(toDeleteIndex, koso.nodes.size - 1)) || null;
      }
    }
  }

  function edit() {
    if (!koso.selected) return;
    getRow(koso.selected).edit(true);
  }

  function unselect() {
    koso.selected = null;
  }

  function moveUp() {
    if (!koso.selected) return;
    koso.moveNodeUp(koso.selected);
  }

  function moveDown() {
    if (!koso.selected) return;
    koso.moveNodeDown(koso.selected);
  }

  function moveStart() {
    if (!koso.selected) return;
    koso.moveNodeUpBoundary(koso.selected);
  }

  function moveEnd() {
    if (!koso.selected) return;
    koso.moveNodeDownBoundary(koso.selected);
  }

  function indent() {
    if (!koso.selected) return;
    koso.indentNode(koso.selected);
  }

  function undent() {
    if (!koso.selected) return;
    koso.undentNode(koso.selected);
  }

  function expand() {
    if (!koso.selected) return;
    koso.expand(koso.selected);
  }

  function collapse() {
    if (!koso.selected) return;
    koso.collapse(koso.selected);
  }

  function showDoneTasks() {
    koso.showDone = true;
  }

  function hideDoneTasks() {
    koso.showDone = false;
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

  function selectNextLink() {
    if (koso.selected) {
      const next = koso.getNextLink(koso.selected);
      if (next) {
        koso.selected = next;
      }
    }
  }

  function selectPrevLink() {
    if (koso.selected) {
      const prev = koso.getPrevLink(koso.selected);
      if (prev) {
        koso.selected = prev;
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

  function organizeTasks() {
    if (!koso.selected) return;
    koso.organizeTasks(koso.selected);
  }

  const insertAction: Action<ActionID> = {
    id: "Insert",
    callback: insert,
    title: "Add",
    description: "Add or insert a new task",
    icon: ListPlus,
    shortcut: INSERT_NODE,
    enabled: () =>
      !inboxView &&
      (!koso.selected || koso.canInsert(koso.selected.parent.name)),
  };

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

  const searchAction = new Action({
    id: "Search",
    callback: showSearchPalette,
    description: "Show the search palette",
    icon: Search,
    enabled: () => !inboxView,
    shortcut: new Shortcut({ key: "p", meta: true }),
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
      id: "Expand",
      callback: expand,
      description: "Expand the current task",
      icon: ChevronsUpDown,
      enabled: () =>
        !inboxView && !!koso.selected && koso.canExpand(koso.selected),
      shortcut: new Shortcut({ key: "ArrowRight" }),
    }),
    new Action({
      id: "Collapse",
      callback: collapse,
      description: "Collapse the current task",
      icon: ChevronsDownUp,
      enabled: () =>
        !inboxView && !!koso.selected && koso.canCollapse(koso.selected),
      shortcut: new Shortcut({ key: "ArrowLeft" }),
    }),
    new Action({
      id: "ExpandAll",
      callback: () => koso.expandAll(),
      title: "Expand All",
      description: "Expand all tasks",
      icon: ChevronsUpDown,
      enabled: () => !inboxView,
    }),
    new Action({
      id: "CollapseAll",
      callback: () => koso.collapseAll(),
      title: "Collapse All",
      description: "Collapse all tasks",
      icon: ChevronsDownUp,
      enabled: () => !inboxView,
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
        !inboxView &&
        !!koso.selected &&
        koso.canInsert(koso.selected.parent.name),
    }),
    new Action({
      id: "InsertSubtask",
      callback: insertChild,
      title: "Insert subtask",
      description: "Insert a new task as a child",
      icon: ListTree,
      enabled: () =>
        !inboxView && !!koso.selected && koso.canInsert(koso.selected.name),
      shortcut: INSERT_CHILD_NODE,
    }),
    new Action({
      id: "InsertSubtaskAbove",
      callback: insertChildAbove,
      title: "Insert subtask above",
      description: "Insert a new task as a child of the previous task",
      icon: ListTree,
      enabled: () => {
        if (inboxView || !koso.selected) {
          return false;
        }
        const prevPeer = koso.getPrevPeer(koso.selected);
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
      enabled: () => !!koso.selected && koso.isEditable(koso.selected.name),
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
        !inboxView && !!koso.selected && koso.canDeleteNode(koso.selected),
      shortcut: new Shortcut({ key: "Delete" }),
    }),
    new Action({
      id: "MoveUp",
      callback: moveUp,
      title: "Move up",
      description: "Move the current task up",
      icon: MoveUp,
      enabled: () => !inboxView && !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowUp", alt: true }),
    }),
    new Action({
      id: "MoveDown",
      callback: moveDown,
      title: "Move down",
      description: "Move the current task down",
      icon: MoveDown,
      enabled: () => !inboxView && !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowDown", alt: true }),
    }),
    new Action({
      id: "MoveToStart",
      callback: moveStart,
      title: "Move to start",
      description: "Move the current task to the top of its group",
      icon: ListStart,
      enabled: () => !inboxView && !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowUp", alt: true, shift: true }),
    }),
    new Action({
      id: "MoveToEnd",
      callback: moveEnd,
      title: "Move to end",
      description: "Move the current task to the bottom of its group",
      icon: ListEnd,
      enabled: () => !inboxView && !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowDown", alt: true, shift: true }),
    }),
    new Action({
      id: "Undent",
      callback: undent,
      title: "Unindent task",
      description: "Make the current task a peer of its parent",
      icon: IndentDecrease,
      enabled: () =>
        !inboxView && !!koso.selected && koso.canUndentNode(koso.selected),
      shortcut: new Shortcut({ key: "ArrowLeft", alt: true }),
    }),
    new Action({
      id: "Indent",
      callback: indent,
      title: "Indent task",
      description: "Make the current task a child of its peer",
      icon: IndentIncrease,
      enabled: () =>
        !inboxView && !!koso.selected && koso.canIndentNode(koso.selected),
      shortcut: new Shortcut({ key: "ArrowRight", alt: true }),
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
      id: "HideDoneTasks",
      callback: hideDoneTasks,
      title: "Hide Done Tasks",
      description: "Hide tasks that have been marked done",
      icon: EyeOff,
      enabled: () => !inboxView && koso.showDone,
    }),
    new Action({
      id: "ShowDoneTasks",
      callback: showDoneTasks,
      title: "Show Done Tasks",
      description: "Show tasks that have been marked done",
      icon: Eye,
      enabled: () => !inboxView && !koso.showDone,
    }),
    searchAction,
    new Action({
      id: "NextLink",
      callback: selectNextLink,
      title: "Next Link",
      description: "Select next link to current task",
      icon: SkipForward,
      enabled: () => !inboxView && !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowDown", meta: true }),
    }),
    new Action({
      id: "PreviousLink",
      callback: selectPrevLink,
      title: "Previous Link",
      description: "Select previous link to current task",
      icon: SkipBack,
      enabled: () => !inboxView && !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowUp", meta: true }),
    }),
  ];

  if (inboxView) {
    actions.push(
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
    );
  } else {
    actions.push(
      new Action({
        id: "Link",
        callback: linkTask,
        title: "Link task to...",
        description: "Link current task to another task",
        icon: Cable,
        enabled: () => !!koso.selected,
        shortcut: new Shortcut({ key: "/", meta: true }),
      }),
      new Action({
        id: "Block",
        callback: blockTask,
        title: "Block task on...",
        description: "Block current task to another task",
        icon: OctagonX,
        enabled: () => !!koso.selected,
      }),
      new Action({
        id: "Organize",
        callback: organizeTasks,
        title: "Organize Tasks",
        description: "Organize the current task and its peers",
        icon: Wrench,
        enabled: () => !!koso.selected,
      }),
    );
  }

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

<SearchPanel bind:open={searchPaletteOpen} />

<Toolbar
  actions={inboxView
    ? [undoAction, redoAction]
    : [insertAction, undoAction, redoAction, searchAction]}
>
  {#await koso.synced then}
    {#if koso.nodes.size > 1}
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
            {#if !inboxView}
              <th class="border-l p-2 max-md:hidden">Reporter</th>
            {/if}
            <th class="relative m-0 w-0 p-0"></th>
          </tr>
        </thead>

        {#each [...koso.nodes].slice(1) as node, index (node.id)}
          <tbody animate:flip={{ duration: 250 }}>
            <!-- eslint-disable-next-line svelte/no-unused-svelte-ignore -->
            <!-- svelte-ignore binding_property_non_reactive -->
            <Row bind:this={rows[node.id]} {index} {node} {users} {inboxView} />
          </tbody>
        {/each}
      </table>
    {:else if !inboxView}
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
