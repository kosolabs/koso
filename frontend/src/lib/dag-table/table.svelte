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
  import MarkdownEditor from "./markdown-editor.svelte";
  import Row from "./row.svelte";
  import SearchPanel from "./search-panel.svelte";
  import Toolbar from "./toolbar.svelte";
  import { ProjectContext } from "./project-context.svelte";

  type Props = {
    users: User[];
    inboxView: boolean;
    projectCtx: ProjectContext;
  };
  const { users, inboxView, projectCtx }: Props = $props();

  const rows: { [key: string]: Row } = {};

  const koso = projectCtx.koso;

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
    projectCtx.selected = node;

    // The newly inserted node's row won't yet have been inserted into
    // the dom and thus onMount will not have been called to register
    // row callbacks.
    // Delay interacting with the row registry to start editing.
    tick().then(() => getRow(node).edit(true));
  }

  function insert() {
    if (projectCtx.selected) {
      insertAndEdit(
        projectCtx.selected.parent,
        koso.getOffset(projectCtx.selected) + 1,
        auth.user,
      );
    } else {
      insertAndEdit(koso.root, 0, auth.user);
    }
  }

  function insertAbove() {
    if (!projectCtx.selected) return;
    insertAndEdit(
      projectCtx.selected.parent,
      koso.getOffset(projectCtx.selected),
      auth.user,
    );
  }

  function insertChild() {
    if (!projectCtx.selected) return;
    projectCtx.expand(projectCtx.selected);
    insertAndEdit(projectCtx.selected, 0, auth.user);
  }

  function insertChildAbove() {
    if (!projectCtx.selected) return;

    const previousPeer = projectCtx.getPrevPeer(projectCtx.selected);
    if (!previousPeer) return;

    projectCtx.expand(previousPeer);
    const lastIndex = koso.getChildCount(previousPeer.name);
    insertAndEdit(previousPeer, lastIndex, auth.user);
  }

  function toggleStatus() {
    if (!projectCtx.selected) return;

    const task = koso.getTask(projectCtx.selected.name);
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
        koso.setTaskStatus(projectCtx.selected.name, "Not Started", auth.user);
        return;
      case "In Progress": {
        const node = projectCtx.selected;

        getRow(node).showDoneConfetti();
        koso.setTaskStatus(node.name, "Done", auth.user);
        if (inboxView) {
          toast.success("🚀 Great work! Task complete!");
        }
        break;
      }
      case "Not Started":
        koso.setTaskStatus(projectCtx.selected.name, "In Progress", auth.user);
        break;
      default:
        throw new Error(`Unhandled status ${task.yStatus}`);
    }
  }

  function remove() {
    if (!projectCtx.selected) return;
    const toDelete = projectCtx.selected;
    const toDeleteIndex = projectCtx.nodes.indexOf(toDelete);

    koso.deleteNode(toDelete);

    if (!inboxView) {
      // Select the next (or previous) node following deletion.
      if (projectCtx.nodes.size < 2 || toDeleteIndex <= 0) {
        projectCtx.selected = null;
      } else {
        projectCtx.selected =
          projectCtx.nodes.get(
            Math.min(toDeleteIndex, projectCtx.nodes.size - 1),
          ) || null;
      }
    }
  }

  function edit() {
    if (!projectCtx.selected) return;
    getRow(projectCtx.selected).edit(true);
  }

  function unselect() {
    projectCtx.selected = null;
  }

  function moveUp() {
    if (!projectCtx.selected) return;
    projectCtx.moveNodeUp(projectCtx.selected);
  }

  function moveDown() {
    if (!projectCtx.selected) return;
    projectCtx.moveNodeDown(projectCtx.selected);
  }

  function moveStart() {
    if (!projectCtx.selected) return;
    projectCtx.moveNodeUpBoundary(projectCtx.selected);
  }

  function moveEnd() {
    if (!projectCtx.selected) return;
    projectCtx.moveNodeDownBoundary(projectCtx.selected);
  }

  function indent() {
    if (!projectCtx.selected) return;
    projectCtx.indentNode(projectCtx.selected);
  }

  function undent() {
    if (!projectCtx.selected) return;
    projectCtx.undentNode(projectCtx.selected);
  }

  function expand() {
    if (!projectCtx.selected) return;
    projectCtx.expand(projectCtx.selected);
  }

  function collapse() {
    if (!projectCtx.selected) return;
    projectCtx.collapse(projectCtx.selected);
  }

  function showDoneTasks() {
    projectCtx.showDone = true;
  }

  function hideDoneTasks() {
    projectCtx.showDone = false;
  }

  function selectNext() {
    if (projectCtx.nodes.size > 1) {
      if (projectCtx.selected) {
        const selectedIndex = projectCtx.nodes.indexOf(projectCtx.selected);
        if (selectedIndex <= 0) {
          projectCtx.selected = null;
        } else {
          const index = Math.min(selectedIndex + 1, projectCtx.nodes.size - 1);
          projectCtx.selected = projectCtx.nodes.get(index, null);
        }
      } else {
        projectCtx.selected = projectCtx.nodes.get(1, null);
      }
    }
  }

  function selectPrev() {
    if (projectCtx.nodes.size > 1) {
      if (projectCtx.selected) {
        const selectedIndex = projectCtx.nodes.indexOf(projectCtx.selected);
        if (selectedIndex <= 0) {
          projectCtx.selected = null;
        } else {
          const index = Math.max(selectedIndex - 1, 1);
          projectCtx.selected = projectCtx.nodes.get(index, null);
        }
      } else {
        projectCtx.selected = projectCtx.nodes.get(
          projectCtx.nodes.size - 1,
          null,
        );
      }
    }
  }

  function selectNextLink() {
    if (projectCtx.selected) {
      const next = projectCtx.getNextLink(projectCtx.selected);
      if (next) {
        projectCtx.selected = next;
      }
    }
  }

  function selectPrevLink() {
    if (projectCtx.selected) {
      const prev = projectCtx.getPrevLink(projectCtx.selected);
      if (prev) {
        projectCtx.selected = prev;
      }
    }
  }

  function undo() {
    projectCtx.undo();
  }

  function redo() {
    projectCtx.redo();
  }

  function linkTask() {
    if (!projectCtx.selected) return;
    getRow(projectCtx.selected).linkPanel(true, "link");
  }

  function blockTask() {
    if (!projectCtx.selected) return;
    getRow(projectCtx.selected).linkPanel(true, "block");
  }

  function organizeTasks() {
    if (!projectCtx.selected) return;
    koso.organizeTasks(projectCtx.selected.parent.name);
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
      (!projectCtx.selected || koso.canInsert(projectCtx.selected.parent.name)),
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
        !inboxView &&
        !!projectCtx.selected &&
        projectCtx.canExpand(projectCtx.selected),
      shortcut: new Shortcut({ key: "ArrowRight" }),
    }),
    new Action({
      id: "Collapse",
      callback: collapse,
      description: "Collapse the current task",
      icon: ChevronsDownUp,
      enabled: () =>
        !inboxView &&
        !!projectCtx.selected &&
        projectCtx.canCollapse(projectCtx.selected),
      shortcut: new Shortcut({ key: "ArrowLeft" }),
    }),
    new Action({
      id: "ExpandAll",
      callback: () => projectCtx.expandAll(),
      title: "Expand All",
      description: "Expand all tasks",
      icon: ChevronsUpDown,
      enabled: () => !inboxView,
    }),
    new Action({
      id: "CollapseAll",
      callback: () => projectCtx.collapseAll(),
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
        !!projectCtx.selected &&
        koso.canInsert(projectCtx.selected.parent.name),
    }),
    new Action({
      id: "InsertSubtask",
      callback: insertChild,
      title: "Insert subtask",
      description: "Insert a new task as a child",
      icon: ListTree,
      enabled: () =>
        !inboxView &&
        !!projectCtx.selected &&
        koso.canInsert(projectCtx.selected.name),
      shortcut: INSERT_CHILD_NODE,
    }),
    new Action({
      id: "InsertSubtaskAbove",
      callback: insertChildAbove,
      title: "Insert subtask above",
      description: "Insert a new task as a child of the previous task",
      icon: ListTree,
      enabled: () => {
        if (inboxView || !projectCtx.selected) {
          return false;
        }
        const prevPeer = projectCtx.getPrevPeer(projectCtx.selected);
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
        !!projectCtx.selected && koso.isEditable(projectCtx.selected.name),
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
        !inboxView &&
        !!projectCtx.selected &&
        koso.canDeleteNode(projectCtx.selected),
      shortcut: new Shortcut({ key: "Delete" }),
    }),
    new Action({
      id: "MoveUp",
      callback: moveUp,
      title: "Move up",
      description: "Move the current task up",
      icon: MoveUp,
      enabled: () => !inboxView && !!projectCtx.selected,
      shortcut: new Shortcut({ key: "ArrowUp", alt: true }),
    }),
    new Action({
      id: "MoveDown",
      callback: moveDown,
      title: "Move down",
      description: "Move the current task down",
      icon: MoveDown,
      enabled: () => !inboxView && !!projectCtx.selected,
      shortcut: new Shortcut({ key: "ArrowDown", alt: true }),
    }),
    new Action({
      id: "MoveToStart",
      callback: moveStart,
      title: "Move to start",
      description: "Move the current task to the top of its group",
      icon: ListStart,
      enabled: () => !inboxView && !!projectCtx.selected,
      shortcut: new Shortcut({ key: "ArrowUp", alt: true, shift: true }),
    }),
    new Action({
      id: "MoveToEnd",
      callback: moveEnd,
      title: "Move to end",
      description: "Move the current task to the bottom of its group",
      icon: ListEnd,
      enabled: () => !inboxView && !!projectCtx.selected,
      shortcut: new Shortcut({ key: "ArrowDown", alt: true, shift: true }),
    }),
    new Action({
      id: "Undent",
      callback: undent,
      title: "Unindent task",
      description: "Make the current task a peer of its parent",
      icon: IndentDecrease,
      enabled: () =>
        !inboxView &&
        !!projectCtx.selected &&
        projectCtx.canUndentNode(projectCtx.selected),
      shortcut: new Shortcut({ key: "ArrowLeft", alt: true }),
    }),
    new Action({
      id: "Indent",
      callback: indent,
      title: "Indent task",
      description: "Make the current task a child of its peer",
      icon: IndentIncrease,
      enabled: () =>
        !inboxView &&
        !!projectCtx.selected &&
        projectCtx.canIndentNode(projectCtx.selected),
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
      enabled: () =>
        !!projectCtx.selected && koso.isEditable(projectCtx.selected.name),
    }),
    new Action({
      id: "HideDoneTasks",
      callback: hideDoneTasks,
      title: "Hide Done Tasks",
      description: "Hide tasks that have been marked done",
      icon: EyeOff,
      enabled: () => !inboxView && projectCtx.showDone,
    }),
    new Action({
      id: "ShowDoneTasks",
      callback: showDoneTasks,
      title: "Show Done Tasks",
      description: "Show tasks that have been marked done",
      icon: Eye,
      enabled: () => !inboxView && !projectCtx.showDone,
    }),
    searchAction,
    new Action({
      id: "NextLink",
      callback: selectNextLink,
      title: "Next Link",
      description: "Select next link to current task",
      icon: SkipForward,
      enabled: () => !inboxView && !!projectCtx.selected,
      shortcut: new Shortcut({ key: "ArrowDown", meta: true }),
    }),
    new Action({
      id: "PreviousLink",
      callback: selectPrevLink,
      title: "Previous Link",
      description: "Select previous link to current task",
      icon: SkipBack,
      enabled: () => !inboxView && !!projectCtx.selected,
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
        enabled: () => !!projectCtx.selected,
      }),
      new Action({
        id: "Block",
        callback: blockTask,
        title: "Block task on...",
        description: "Block current task to another task",
        icon: OctagonX,
        enabled: () => !!projectCtx.selected,
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
        enabled: () => !!projectCtx.selected,
        shortcut: new Shortcut({ key: "/", meta: true }),
      }),
      new Action({
        id: "Block",
        callback: blockTask,
        title: "Block task on...",
        description: "Block current task to another task",
        icon: OctagonX,
        enabled: () => !!projectCtx.selected,
      }),
      new Action({
        id: "Organize",
        callback: organizeTasks,
        title: "Organize Tasks",
        description: "Organize the current task and its peers",
        icon: Wrench,
        enabled: () => !!projectCtx.selected,
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
      projectCtx.select(taskId);
    } else {
      projectCtx.selected = null;
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
    const selected = projectCtx.selectedRaw;
    const node = selected.node;
    const index = selected.index;

    if (!node) {
      return;
    }

    const currentIndex = projectCtx.nodes.indexOf(node);
    if (currentIndex !== -1) {
      // The node still exists. Make sure the stashed index still matches.
      if (!index || index !== currentIndex) {
        console.debug(
          `Refreshing selected index for node ${node.id} at prior index ${index}`,
        );
        projectCtx.selected = node;
      }
      return;
    }

    // The selected node no longer exists. Select the
    // node at the same index or the one at the end of the list.
    // The first node is not selectable.
    if (projectCtx.nodes.size > 1) {
      console.debug(`Node ${node.id} no longer exists. Selecting new node.`);
      projectCtx.selected = projectCtx.nodes.get(
        Math.min(index || -1, projectCtx.nodes.size - 1),
        null,
      );
    } else {
      console.debug(`Node ${node.id} no longer exists. Clearing selection.`);
      projectCtx.selected = null;
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
    {#if projectCtx.nodes.size > 1}
      <MarkdownEditor taskId={projectCtx.selected?.name} />

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

        {#each [...projectCtx.nodes].slice(1) as node, index (node.id)}
          <tbody animate:flip={{ duration: 250 }}>
            <!-- eslint-disable-next-line svelte/no-unused-svelte-ignore -->
            <!-- svelte-ignore binding_property_non_reactive -->
            <Row
              bind:this={rows[node.id]}
              {index}
              {node}
              {users}
              {inboxView}
              {projectCtx}
            />
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
