<script lang="ts">
  import kosoLogo from "$lib/assets/koso.svg";
  import { user, type User } from "$lib/auth";
  import { Button } from "$lib/components/ui/button";
  import { CommandPalette } from "$lib/components/ui/command-palette";
  import { Node, type Koso } from "$lib/koso";
  import { Shortcut, ShortcutRegistry, type Action } from "$lib/shortcuts";
  import {
    Check,
    ChevronsDownUp,
    ChevronsUpDown,
    CircleX,
    Eye,
    EyeOff,
    IndentDecrease,
    IndentIncrease,
    ListPlus,
    ListTree,
    MoveDown,
    MoveUp,
    Pencil,
    Redo,
    SquarePen,
    StepBack,
    StepForward,
    Sun,
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
  };
  const { koso, users }: Props = $props();
  const { debug, nodes, selected, showDone, syncState } = koso;

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
    if (!$user) throw new Error("Unauthenticated");
    if ($selected) {
      insertAndEdit($selected.parent, koso.getOffset($selected) + 1, $user);
    } else {
      insertAndEdit(koso.root, 0, $user);
    }
  }

  function insertChild() {
    if (!$selected) return;
    if (!$user) throw new Error("Unauthenticated");
    koso.expand($selected);
    insertAndEdit($selected, 0, $user);
  }

  function toggleStatus() {
    if (!$selected) return;
    if (!$user) throw new Error("Unauthenticated");

    const task = koso.getTask($selected.name);
    if (task.children.length > 0) {
      toast.warning(
        "Cannot change the status of a composite task. Change the status of the task's children instead.",
      );
      return;
    }

    if (task.status === "Done") {
      return;
    } else if (task.status === "In Progress") {
      koso.setTaskStatus($selected, "Done", $user);
    } else {
      koso.setTaskStatus($selected, "In Progress", $user);
    }
  }

  function remove() {
    if (!$selected) return;
    const toDelete = $selected;
    const toDeleteIndex = $nodes.indexOf(toDelete);

    koso.deleteNode(toDelete);

    // Select the next (or previous) node following deletion.
    if ($nodes.size < 2) {
      $selected = null;
    } else {
      $selected = $nodes.get(Math.min(toDeleteIndex, $nodes.size - 1)) || null;
    }
  }

  function edit() {
    if (!$selected) return;
    getRow($selected).edit(true);
  }

  function unselect() {
    $selected = null;
  }

  function moveUp() {
    if (!$selected) return;
    koso.moveNodeUp($selected);
  }

  function moveDown() {
    if (!$selected) return;
    koso.moveNodeDown($selected);
  }

  function moveRowUp() {
    if (!$selected) return;
    koso.moveNodeRowUp($selected);
  }

  function moveRowDown() {
    if (!$selected) return;
    koso.moveNodeRowDown($selected);
  }

  function indent() {
    if (!$selected) return;
    koso.indentNode($selected);
  }

  function undent() {
    if (!$selected) return;
    koso.undentNode($selected);
  }

  function expand() {
    if (!$selected) return;
    koso.expand($selected);
  }

  function collapse() {
    if (!$selected) return;
    koso.collapse($selected);
  }

  function showDoneTasks() {
    koso.setShowDone(true);
  }

  function hideDoneTasks() {
    koso.setShowDone(false);
  }

  function selectNext() {
    if ($nodes.size > 1) {
      if ($selected) {
        $nodes.indexOf($selected);
        const index = Math.min($nodes.indexOf($selected) + 1, $nodes.size - 1);
        $selected = $nodes.get(index, null);
      } else {
        $selected = $nodes.get(1, null);
      }
    }
  }

  function selectPrev() {
    if ($nodes.size > 1) {
      if ($selected) {
        const index = Math.max($nodes.indexOf($selected) - 1, 1);
        $selected = $nodes.get(index, null);
      } else {
        $selected = $nodes.get($nodes.size - 1, null);
      }
    }
  }

  function undo() {
    koso.undo();
  }

  function redo() {
    koso.redo();
  }

  const actions: Action[] = [
    {
      title: "Add Task",
      icon: ListPlus,
      callback: insert,
      toolbar: true,
      shortcut: Shortcut.INSERT_NODE,
    },
    {
      title: "Edit Name",
      icon: Pencil,
      callback: edit,
      shortcut: Shortcut.EDIT_NODE,
    },
    {
      title: "Cancel Selection",
      icon: CircleX,
      callback: unselect,
      shortcut: Shortcut.CANCEL,
    },
    {
      title: "Add Child",
      icon: ListTree,
      callback: insertChild,
      toolbar: true,
      enabled: () => !!$selected,
      shortcut: Shortcut.INSERT_CHILD_NODE,
    },
    {
      title: "Delete",
      icon: Trash,
      callback: remove,
      toolbar: true,
      enabled: () => !!$selected,
      shortcut: Shortcut.REMOVE_NODE,
    },
    {
      title: "Move Up",
      icon: MoveUp,
      callback: moveUp,
      toolbar: true,
      enabled: () => !!$selected,
      shortcut: Shortcut.MOVE_NODE_UP,
    },
    {
      title: "Move Down",
      icon: MoveDown,
      callback: moveDown,
      toolbar: true,
      enabled: () => !!$selected,
      shortcut: Shortcut.MOVE_NODE_DOWN,
    },
    {
      title: "Move Row Up",
      icon: MoveUp,
      callback: moveRowUp,
      enabled: () => !!$selected,
      shortcut: Shortcut.MOVE_NODE_ROW_UP,
    },
    {
      title: "Move Row Down",
      icon: MoveDown,
      callback: moveRowDown,
      enabled: () => !!$selected,
      shortcut: Shortcut.MOVE_NODE_ROW_DOWN,
    },
    {
      title: "Undent",
      icon: IndentDecrease,
      callback: undent,
      toolbar: true,
      enabled: () => !!$selected,
      shortcut: Shortcut.UNDENT_NODE,
    },
    {
      title: "Indent",
      icon: IndentIncrease,
      callback: indent,
      toolbar: true,
      enabled: () => !!$selected,
      shortcut: Shortcut.INDENT_NODE,
    },
    {
      title: "Undent",
      icon: IndentDecrease,
      callback: undent,
      enabled: () => !!$selected,
      shortcut: Shortcut.UNDENT_NODE_SHIFT,
    },
    {
      title: "Indent",
      icon: IndentIncrease,
      callback: indent,
      enabled: () => !!$selected,
      shortcut: Shortcut.INDENT_NODE_SHIFT,
    },
    {
      title: "Undo",
      icon: Undo,
      callback: undo,
      toolbar: true,
      shortcut: Shortcut.UNDO,
    },
    {
      title: "Redo",
      icon: Redo,
      callback: redo,
      toolbar: true,
      shortcut: Shortcut.REDO,
    },
    {
      title: "Hide Done Tasks",
      icon: EyeOff,
      callback: hideDoneTasks,
      enabled: () => !!$showDone,
      toolbar: true,
    },
    {
      title: "Show Done Tasks",
      icon: Eye,
      callback: showDoneTasks,
      enabled: () => !$showDone,
      toolbar: true,
    },
    {
      title: "Expand Task",
      icon: ChevronsUpDown,
      callback: expand,
      shortcut: Shortcut.EXPAND_NODE,
    },
    {
      title: "Collapse Task",
      icon: ChevronsDownUp,
      callback: collapse,
      shortcut: Shortcut.COLLAPSE_NODE,
    },
    {
      title: "Select Next Task",
      icon: StepForward,
      callback: selectNext,
      shortcut: Shortcut.SELECT_NEXT_NODE,
    },
    {
      title: "Select Previous Task",
      icon: StepBack,
      callback: selectPrev,
      shortcut: Shortcut.SELECT_PREV_NODE,
    },
    {
      title: "Toggle Task Status",
      icon: Check,
      callback: toggleStatus,
      shortcut: Shortcut.TOGGLE_STATUS,
    },
    {
      title: "Set Theme to Light",
      icon: Sun,
      callback: () => setMode("light"),
    },
    {
      title: "Set Theme to Dark",
      icon: Sun,
      callback: () => setMode("dark"),
    },
    {
      title: "Set Theme to System",
      icon: Sun,
      callback: () => resetMode(),
    },
    {
      title: "Show Command Palette",
      icon: Terminal,
      callback: showCommandPalette,
      toolbar: true,
      shortcut: Shortcut.SHOW_COMMAND_PALETTE,
    },
  ];

  const shortcutRegistry = new ShortcutRegistry(actions);

  onMount(() => {
    const keyDownListener = (event: KeyboardEvent) => {
      if ($debug) {
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

<Toolbar {actions} />
<CommandPalette bind:open={commandPaletteOpen} {actions} />

<div class="mb-12 p-2 sm:mb-0">
  {#if !$syncState.serverSync && !$syncState.indexedDbSync}
    <!-- Loading.-->
  {:else if $nodes.size > 1}
    <table class="w-full border-separate border-spacing-0 rounded-md border">
      <thead class="text-left text-xs font-bold uppercase">
        <tr>
          <th class="w-32 p-2">ID</th>
          {#if $debug}
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

      {#each [...$nodes].slice(1) as node, index (node.id)}
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
          <img class="size-16" alt="Koso Logo" src={kosoLogo} />
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
</div>
