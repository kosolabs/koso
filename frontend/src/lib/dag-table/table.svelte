<script lang="ts">
  import { auth, type User } from "$lib/auth.svelte";
  import { Button } from "$lib/components/ui/button";
  import { CommandPalette } from "$lib/components/ui/command-palette";
  import { confetti } from "$lib/components/ui/confetti";
  import KosoLogo from "$lib/components/ui/koso-logo/koso-logo.svelte";
  import { Node, type Koso } from "$lib/koso.svelte";
  import { Action, Shortcut, ShortcutRegistry } from "$lib/shortcuts";
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
  import { toast } from "svelte-sonner";
  import { flip } from "svelte/animate";
  import Row, { type RowType } from "./row.svelte";
  import Toolbar from "./toolbar.svelte";

  type Props = {
    koso: Koso;
    users: User[];
    extraActions: Action[];
  };
  const { koso, users, extraActions }: Props = $props();

  const rows: { [key: string]: RowType } = {};

  function getRow(node: Node) {
    const maybeRow = rows[node.id];
    if (!maybeRow) {
      throw new Error(`Row doesn't exist for ${node}`);
    }
    return maybeRow;
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
    if (task.children.length > 0) {
      toast.warning(
        "Cannot change the status of a composite task. Change the status of the task's children instead.",
      );
      return;
    }

    if (task.status === "Done") {
      return;
    } else if (task.status === "In Progress") {
      confetti.add(getRow(koso.selected).getStatusPosition());
      koso.setTaskStatus(koso.selected, "Done", auth.user);
    } else {
      koso.setTaskStatus(koso.selected, "In Progress", auth.user);
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
      enabled: () => !!koso.selected && koso.canExpand(koso.selected),
      shortcut: new Shortcut({ key: "ArrowRight" }),
    }),
    new Action({
      callback: collapse,
      title: "Collapse",
      description: "Collapse the current task",
      icon: ChevronsDownUp,
      enabled: () => !!koso.selected && koso.canCollapse(koso.selected),
      shortcut: new Shortcut({ key: "ArrowLeft" }),
    }),
    new Action({
      callback: insert,
      title: "Insert",
      description: "Insert a new task",
      icon: ListPlus,
      toolbar: true,
      shortcut: Shortcut.INSERT_NODE,
    }),
    new Action({
      callback: insertAbove,
      title: "Insert Above",
      description: "Insert a new task above",
      icon: ListPlus,
      shortcut: new Shortcut({ key: "Enter", meta: true, shift: true }),
    }),
    new Action({
      callback: insertChild,
      title: "Insert Subtask",
      description: "Insert a new task as a child",
      icon: ListTree,
      enabled: () => !!koso.selected,
      shortcut: Shortcut.INSERT_CHILD_NODE,
    }),
    new Action({
      callback: insertChildAbove,
      title: "Insert Subtask Above",
      description: "Insert a new task as a child of the previous task",
      icon: ListTree,
      enabled: () => !!koso.selected && koso.getOffset(koso.selected) > 0,
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
    }),
    new Action({
      callback: unselect,
      title: "Clear",
      description: "Clear the current selection",
      icon: CircleX,
      shortcut: Shortcut.CANCEL,
    }),
    new Action({
      callback: remove,
      title: "Delete",
      description: "Delete the current task",
      icon: Trash,
      toolbar: true,
      enabled: () => !!koso.selected,
      shortcut: new Shortcut({ key: "Delete" }),
    }),
    new Action({
      callback: moveUp,
      title: "Move Up",
      description: "Move the current task up",
      icon: MoveUp,
      toolbar: true,
      enabled: () => !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowUp", alt: true }),
    }),
    new Action({
      callback: moveDown,
      title: "Move Down",
      description: "Move the current task down",
      icon: MoveDown,
      toolbar: true,
      enabled: () => !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowDown", alt: true }),
    }),
    new Action({
      callback: moveStart,
      title: "Move to Start",
      description: "Move the current task to the top of its group",
      icon: ListStart,
      toolbar: true,
      enabled: () => !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowUp", alt: true, shift: true }),
    }),
    new Action({
      callback: moveEnd,
      title: "Move to End",
      description: "Move the current task to the bottom of its group",
      icon: ListEnd,
      toolbar: true,
      enabled: () => !!koso.selected,
      shortcut: new Shortcut({ key: "ArrowDown", alt: true, shift: true }),
    }),
    new Action({
      callback: undent,
      title: "Undent",
      description: "Make the current task a peer of its parent",
      icon: IndentDecrease,
      toolbar: true,
      enabled: () => !!koso.selected && koso.canUndentNode(koso.selected),
      shortcut: new Shortcut({ key: "ArrowLeft", alt: true }),
    }),
    new Action({
      callback: indent,
      title: "Indent",
      description: "Make the current task a child of its peer",
      icon: IndentIncrease,
      toolbar: true,
      enabled: () => !!koso.selected && koso.canIndentNode(koso.selected),
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
    }),
    new Action({
      callback: hideDoneTasks,
      title: "Hide Done Tasks",
      description: "Hide tasks that have been marked done",
      icon: EyeOff,
      enabled: () => koso.showDone,
    }),
    new Action({
      callback: showDoneTasks,
      title: "Show Done Tasks",
      description: "Show tasks that have been marked done",
      icon: Eye,
      enabled: () => !koso.showDone,
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

    document.addEventListener("keydown", keyDownListener);

    return () => {
      document.removeEventListener("keydown", keyDownListener);
    };
  });

  setContext<Koso>("koso", koso);
</script>

<CommandPalette bind:open={commandPaletteOpen} {actions} />

<Toolbar {actions}>
  {#if !koso.syncState.serverSync && !koso.syncState.indexedDbSync}
    <!-- Loading.-->
  {:else if koso.nodes.size > 1}
    <table class="w-full border-separate border-spacing-0 rounded-md border">
      <thead class="text-left text-xs font-bold uppercase">
        <tr>
          <th class="w-32 p-2">ID</th>
          {#if koso.debug}
            <th class="border-l p-2">UUID</th>
          {/if}
          <th class="border-l p-2">
            <SquarePen class="h-4 sm:hidden" />
            <div class="max-sm:hidden">Status</div></th
          >
          <th class="border-l p-2">Name</th>
          <th class="border-l p-2">
            <UserRoundPlus class="h-4 sm:hidden" />
            <div class="max-sm:hidden">Assignee</div>
          </th>
          <th class="border-l p-2 max-sm:hidden">Reporter</th>
        </tr>
      </thead>

      {#each [...koso.nodes].slice(1) as node, index (node.id)}
        <tbody animate:flip={{ duration: 250 }}>
          <!-- eslint-disable-next-line svelte/no-unused-svelte-ignore -->
          <!-- svelte-ignore binding_property_non_reactive -->
          <Row bind:this={rows[node.id]} {index} {node} {users} />
        </tbody>
      {/each}
    </table>
  {:else}
    <div class="flex items-center justify-center pt-8">
      <div class="flex w-9/12 max-w-[425px] rounded-md border bg-muted p-4">
        <div class="min-w-16">
          <KosoLogo class="size-16" />
        </div>
        <div class="ml-4">
          <div class="text-md">Welcome to Koso!</div>
          <div class="mt-2 text-xs">
            Koso helps you to organize your work and be productive.
          </div>
          <div class="mt-4">
            <Button size="sm" class="text-xs" onclick={insert}>
              <ListPlus class="w-4 sm:me-2" />
              Add task
            </Button>
          </div>
        </div>
      </div>
    </div>
  {/if}
</Toolbar>
