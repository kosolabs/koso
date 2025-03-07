<script lang="ts">
  import { replaceState } from "$app/navigation";
  import { auth, type User } from "$lib/auth.svelte";
  import { CommandPalette } from "$lib/components/ui/command-palette";
  import KosoLogo from "$lib/components/ui/koso-logo/koso-logo.svelte";
  import { toast } from "$lib/components/ui/sonner";
  import { events } from "$lib/kosui";
  import { Button } from "$lib/kosui/button";
  import { Shortcut } from "$lib/kosui/shortcut";
  import {
    Action,
    CANCEL,
    INSERT_CHILD_NODE,
    INSERT_NODE,
    ShortcutRegistry,
  } from "$lib/shortcuts";
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
    Moon,
    MoveDown,
    MoveUp,
    Pencil,
    Redo,
    Search,
    SkipBack,
    SkipForward,
    SquarePen,
    StepBack,
    StepForward,
    Sun,
    SunMoon,
    Terminal,
    Trash,
    Undo,
    UserRoundPlus,
  } from "lucide-svelte";
  import { resetMode, setMode } from "mode-watcher";
  import { onMount, setContext, tick } from "svelte";
  import { flip } from "svelte/animate";
  import { Node, type Koso } from ".";
  import Row from "./row.svelte";
  import SearchPanel from "./search-panel.svelte";
  import Toolbar from "./toolbar.svelte";

  type Props = {
    koso: Koso;
    users: User[];
    extraActions: Action[];
    inboxView: boolean;
  };
  const { koso, users, extraActions, inboxView }: Props = $props();

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

  let commandPaletteOpen: boolean = $state(false);
  function showCommandPalette() {
    commandPaletteOpen = true;
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
        toast.warning(
          "Cannot change the status of a Blocked task. Change the status of the task's children instead.",
        );
        return;
      case "In Progress":
        getRow(koso.selected).showDoneConfetti();
        koso.setTaskStatus(koso.selected, "Done", auth.user);
        break;
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

    // Select the next (or previous) node following deletion.
    if (koso.nodes.size < 2) {
      koso.selected = null;
    } else {
      koso.selected =
        koso.nodes.get(Math.min(toDeleteIndex, koso.nodes.size - 1)) || null;
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
        koso.nodes.indexOf(koso.selected);
        const index = Math.min(
          koso.nodes.indexOf(koso.selected) + 1,
          koso.nodes.size - 1,
        );
        koso.selected = koso.nodes.get(index, null);
      } else {
        koso.selected = koso.nodes.get(1, null);
      }
    }
  }

  function selectPrev() {
    if (koso.nodes.size > 1) {
      if (koso.selected) {
        const index = Math.max(koso.nodes.indexOf(koso.selected) - 1, 1);
        koso.selected = koso.nodes.get(index, null);
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
    getRow(koso.selected).linkPanel(true);
  }

  export const actions: Action[] = [
    new Action({
      callback: selectNext,
      title: "Next",
      description: "Select next task",
      icon: StepForward,
      shortcut: new Shortcut({ key: "ArrowDown" }),
    }),
    new Action({
      callback: selectPrev,
      title: "Previous",
      description: "Select previous task",
      icon: StepBack,
      shortcut: new Shortcut({ key: "ArrowUp" }),
    }),
    new Action({
      callback: expand,
      title: "Expand",
      description: "Expand the current task",
      icon: ChevronsUpDown,
      enabled: () =>
        !inboxView && !!koso.selected && koso.canExpand(koso.selected),
      shortcut: new Shortcut({ key: "ArrowRight" }),
    }),
    new Action({
      callback: collapse,
      title: "Collapse",
      description: "Collapse the current task",
      icon: ChevronsDownUp,
      enabled: () =>
        !inboxView && !!koso.selected && koso.canCollapse(koso.selected),
      shortcut: new Shortcut({ key: "ArrowLeft" }),
    }),
    new Action({
      callback: () => koso.expandAll(),
      title: "Expand All",
      description: "Expand all tasks",
      icon: ChevronsUpDown,
      enabled: () => !inboxView,
    }),
    new Action({
      callback: () => koso.collapseAll(),
      title: "Collapse All",
      description: "Collapse all tasks",
      icon: ChevronsDownUp,
      enabled: () => !inboxView,
    }),
    new Action({
      callback: insert,
      title: "Add",
      description: "Add or insert a new task",
      icon: ListPlus,
      toolbar: true,
      shortcut: INSERT_NODE,
      enabled: () =>
        !inboxView &&
        (!koso.selected || koso.canInsert(koso.selected.parent.name)),
    }),
    new Action({
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
      callback: insertChild,
      title: "Insert Subtask",
      description: "Insert a new task as a child",
      icon: ListTree,
      enabled: () =>
        !inboxView && !!koso.selected && koso.canInsert(koso.selected.name),
      shortcut: INSERT_CHILD_NODE,
    }),
    new Action({
      callback: insertChildAbove,
      title: "Insert Subtask Above",
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
      callback: edit,
      title: "Edit",
      description: "Edit the current task",
      icon: Pencil,
      shortcut: new Shortcut({ key: "Enter" }),
      enabled: () => !!koso.selected && koso.isEditable(koso.selected.name),
    }),
    new Action({
      callback: unselect,
      title: "Clear",
      description: "Clear the current selection",
      icon: CircleX,
      shortcut: CANCEL,
    }),
    new Action({
      callback: remove,
      title: "Delete",
      description: "Delete the current task",
      icon: Trash,
      toolbar: true,
      enabled: () =>
        !!koso.selected &&
        koso.canDeleteNode(koso.selected.name, koso.selected.parent.name),
      shortcut: new Shortcut({ key: "Delete" }),
    }),
    new Action({
      callback: moveUp,
      title: "Move Up",
      description: "Move the current task up",
      icon: MoveUp,
      toolbar: true,
      enabled: () => !inboxView && !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowUp", alt: true }),
    }),
    new Action({
      callback: moveDown,
      title: "Move Down",
      description: "Move the current task down",
      icon: MoveDown,
      toolbar: true,
      enabled: () => !inboxView && !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowDown", alt: true }),
    }),
    new Action({
      callback: moveStart,
      title: "Move to Start",
      description: "Move the current task to the top of its group",
      icon: ListStart,
      toolbar: true,
      enabled: () => !inboxView && !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowUp", alt: true, shift: true }),
    }),
    new Action({
      callback: moveEnd,
      title: "Move to End",
      description: "Move the current task to the bottom of its group",
      icon: ListEnd,
      toolbar: true,
      enabled: () => !inboxView && !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowDown", alt: true, shift: true }),
    }),
    new Action({
      callback: undent,
      title: "Undent",
      description: "Make the current task a peer of its parent",
      icon: IndentDecrease,
      toolbar: true,
      enabled: () =>
        !inboxView && !!koso.selected && koso.canUndentNode(koso.selected),
      shortcut: new Shortcut({ key: "ArrowLeft", alt: true }),
    }),
    new Action({
      callback: indent,
      title: "Indent",
      description: "Make the current task a child of its peer",
      icon: IndentIncrease,
      toolbar: true,
      enabled: () =>
        !inboxView && !!koso.selected && koso.canIndentNode(koso.selected),
      shortcut: new Shortcut({ key: "ArrowRight", alt: true }),
    }),
    new Action({
      callback: undo,
      title: "Undo",
      icon: Undo,
      toolbar: true,
      shortcut: new Shortcut({ key: "z", meta: true }),
    }),
    new Action({
      callback: redo,
      title: "Redo",
      icon: Redo,
      toolbar: true,
      shortcut: new Shortcut({ key: "z", meta: true, shift: true }),
    }),
    new Action({
      callback: toggleStatus,
      title: "Toggle Task Status",
      description: "Toggle the task status to In Progress or Done",
      icon: Check,
      shortcut: new Shortcut({ key: " " }),
      enabled: () => !!koso.selected && koso.isEditable(koso.selected.name),
    }),
    new Action({
      callback: hideDoneTasks,
      title: "Hide Done Tasks",
      description: "Hide tasks that have been marked done",
      icon: EyeOff,
      enabled: () => !inboxView && koso.showDone,
    }),
    new Action({
      callback: showDoneTasks,
      title: "Show Done Tasks",
      description: "Show tasks that have been marked done",
      icon: Eye,
      enabled: () => !inboxView && !koso.showDone,
    }),
    new Action({
      callback: () => setMode("light"),
      title: "Light",
      description: "Set the theme to light mode",
      icon: Sun,
    }),
    new Action({
      callback: () => setMode("dark"),
      title: "Dark",
      description: "Set the theme to dark mode",
      icon: Moon,
    }),
    new Action({
      callback: () => resetMode(),
      title: "System",
      description: "Set the theme to system",
      icon: SunMoon,
    }),
    new Action({
      callback: showSearchPalette,
      title: "Search",
      description: "Show the search palette",
      icon: Search,
      toolbar: true,
      enabled: () => !inboxView,
      shortcut: new Shortcut({ key: "p", meta: true }),
    }),
    new Action({
      callback: showCommandPalette,
      title: "Palette",
      description: "Show the command palette",
      icon: Terminal,
      toolbar: true,
      shortcut: new Shortcut({ key: "p", shift: true, meta: true }),
    }),
    new Action({
      callback: linkTask,
      title: "Link",
      description: "Link current task to another task",
      icon: Cable,
      toolbar: true,
      enabled: () => !!koso.selected,
      shortcut: new Shortcut({ key: "/", meta: true }),
    }),
    new Action({
      callback: selectNextLink,
      title: "Next Link",
      description: "Select next link to current task",
      icon: SkipForward,
      toolbar: false,
      enabled: () => !inboxView && !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowDown", meta: true }),
    }),
    new Action({
      callback: selectPrevLink,
      title: "Previous Link",
      description: "Select previous link to current task",
      icon: SkipBack,
      toolbar: false,
      enabled: () => !inboxView && !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowUp", meta: true }),
    }),
    ...extraActions,
  ];

  const shortcutRegistry = new ShortcutRegistry(actions);

  onMount(() => {
    const keyDownListener = (event: KeyboardEvent) => {
      if (koso.debug) {
        if (["Alt", "Control", "Meta", "Shift"].includes(event.key)) return;
        console.log(Shortcut.fromEvent(event).toString());
      }

      shortcutRegistry.handle(event);
    };

    return events.on("keydown", keyDownListener);
  });

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
</script>

<CommandPalette bind:open={commandPaletteOpen} {actions} />
<SearchPanel bind:open={searchPaletteOpen} />

<Toolbar {actions}>
  {#await koso.synced then}
    {#if koso.nodes.size > 1}
      <table class="rounded-m3 w-full border-separate border-spacing-0 border">
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
        <div class="bg-muted flex w-9/12 max-w-[425px] rounded-md border p-4">
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
        <div class="bg-muted flex w-9/12 max-w-[425px] rounded-md border p-4">
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
