<script lang="ts">
  import { user, type User } from "$lib/auth";
  import {
    CommandPalette,
    type Action,
  } from "$lib/components/ui/command-palette";
  import { Strokes } from "$lib/components/ui/stroke";
  import { ToolbarButton } from "$lib/components/ui/toolbar-button";
  import { KeyBinding } from "$lib/key-binding";
  import { KeyHandlerRegistry } from "$lib/key-handler-registry";
  import { type Koso } from "$lib/koso";
  import {
    globalKeybindingsEnabled,
    handleOpenChange,
  } from "$lib/popover-monitors";
  import { cn } from "$lib/utils";
  import {
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
    Terminal,
    Trash,
    Undo,
    UserRoundPlus,
  } from "lucide-svelte";
  import { setContext } from "svelte";
  import { toast } from "svelte-sonner";
  import { flip } from "svelte/animate";
  import Row from "./row.svelte";

  export let koso: Koso;
  export let users: User[];
  export let commandPaletteOpen: boolean = false;

  const { debug, editing, nodes, selected, showDone } = koso;

  let commandPalette: any;
  $: handleOpenChange(commandPaletteOpen, commandPalette);

  function showCommandPalette() {
    commandPaletteOpen = true;
  }

  function insert() {
    if (!$user) throw new Error("Unauthenticated");
    if ($selected) {
      koso.insertNode($selected.parent, koso.getOffset($selected) + 1, $user);
    } else {
      koso.insertNode(koso.root, 0, $user);
    }
  }

  function insertChild() {
    if (!$selected) return;
    if (!$user) throw new Error("Unauthenticated");
    koso.expand($selected);
    koso.insertNode($selected, 0, $user);
  }

  function remove() {
    if (!$selected) return;
    const toDelete = $selected;

    selectNext();
    if (toDelete.equals($selected)) {
      selectPrev();
    }

    const adjSelected = !toDelete.equals($selected);
    koso.deleteNode(toDelete);
    if (!adjSelected) {
      $selected = null;
    }
  }

  function edit() {
    if (!$selected) return;
    $editing = true;
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

  let actions: Action[];
  $: {
    actions = [];
    actions.push({
      title: "Add Task",
      icon: ListPlus,
      callback: insert,
      toolbar: true,
      shortcut: KeyBinding.INSERT_NODE,
    });
    actions.push({
      title: "Edit Name",
      icon: Pencil,
      callback: edit,
      toolbar: false,
      shortcut: KeyBinding.EDIT_NODE,
    });
    actions.push({
      title: "Cancel Selection",
      icon: CircleX,
      callback: unselect,
      toolbar: false,
      shortcut: KeyBinding.CANCEL_SELECTION,
    });
    if ($selected) {
      actions.push({
        title: "Add Child",
        icon: ListTree,
        callback: insertChild,
        toolbar: true,
        shortcut: KeyBinding.INSERT_CHILD_NODE,
      });
      actions.push({
        title: "Delete",
        icon: Trash,
        callback: remove,
        toolbar: true,
        shortcut: KeyBinding.REMOVE_NODE,
      });
      actions.push({
        title: "Move Up",
        icon: MoveUp,
        callback: moveUp,
        toolbar: true,
        shortcut: KeyBinding.MOVE_NODE_UP,
      });
      actions.push({
        title: "Move Down",
        icon: MoveDown,
        callback: moveDown,
        toolbar: true,
        shortcut: KeyBinding.MOVE_NODE_DOWN,
      });
      actions.push({
        title: "Move Row Up",
        icon: MoveUp,
        callback: moveRowUp,
        toolbar: true,
        shortcut: KeyBinding.MOVE_NODE_ROW_UP,
      });
      actions.push({
        title: "Move Row Down",
        icon: MoveDown,
        callback: moveRowDown,
        toolbar: true,
        shortcut: KeyBinding.MOVE_NODE_ROW_DOWN,
      });
      actions.push({
        title: "Undent",
        icon: IndentDecrease,
        callback: undent,
        toolbar: true,
        shortcut: KeyBinding.UNDENT_NODE,
      });
      actions.push({
        title: "Indent",
        icon: IndentIncrease,
        callback: indent,
        toolbar: true,
        shortcut: KeyBinding.INDENT_NODE,
      });
      actions.push({
        title: "Undent",
        icon: IndentDecrease,
        callback: undent,
        toolbar: true,
        shortcut: KeyBinding.UNDENT_NODE_SHIFT,
      });
      actions.push({
        title: "Indent",
        icon: IndentIncrease,
        callback: indent,
        toolbar: true,
        shortcut: KeyBinding.INDENT_NODE_SHIFT,
      });
    }
    actions.push({
      title: "Undo",
      icon: Undo,
      callback: undo,
      toolbar: true,
      shortcut: KeyBinding.UNDO,
    });
    actions.push({
      title: "Redo",
      icon: Redo,
      callback: redo,
      toolbar: true,
      shortcut: KeyBinding.REDO,
    });
    if ($showDone) {
      actions.push({
        title: "Hide Done Tasks",
        icon: EyeOff,
        callback: hideDoneTasks,
        toolbar: true,
      });
    } else {
      actions.push({
        title: "Show Done Tasks",
        icon: Eye,
        callback: showDoneTasks,
        toolbar: true,
      });
    }
    actions.push({
      title: "Expand Task",
      icon: ChevronsUpDown,
      callback: expand,
      toolbar: false,
      shortcut: KeyBinding.EXPAND_NODE,
    });
    actions.push({
      title: "Collapse Task",
      icon: ChevronsDownUp,
      callback: collapse,
      toolbar: false,
      shortcut: KeyBinding.COLLAPSE_NODE,
    });
    actions.push({
      title: "Select Next Task",
      icon: StepForward,
      callback: selectNext,
      toolbar: false,
      shortcut: KeyBinding.SELECT_NEXT_NODE,
    });
    actions.push({
      title: "Select Previous Task",
      icon: StepBack,
      callback: selectPrev,
      toolbar: false,
      shortcut: KeyBinding.SELECT_PREV_NODE,
    });
    actions.push({
      title: "Show Command Palette",
      icon: Terminal,
      callback: showCommandPalette,
      toolbar: true,
      shortcut: KeyBinding.SHOW_COMMAND_PALETTE,
    });
  }

  $: registry = new KeyHandlerRegistry(
    actions
      .filter((action) => action.shortcut)
      .map((action) => [action.shortcut!, action.callback]),
  );

  document.onkeydown = (event: KeyboardEvent) => {
    if ($debug) {
      // TODO: Remove any once toast support Component type
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      toast.info(Strokes as any, {
        componentProps: { binding: KeyBinding.fromEvent(event) },
      });
    }

    console.log(event);
    if (!globalKeybindingsEnabled()) return;
    registry.handle(event);
  };

  setContext<Koso>("koso", koso);
</script>

<div
  class={cn(
    "z-10 flex items-center overflow-x-scroll px-2 backdrop-blur-sm",
    "fixed bottom-0 left-0 h-12 w-full border-t",
    "sm:sticky sm:top-0 sm:gap-2 sm:border-b",
  )}
>
  {#each actions as { title, icon, callback, toolbar }}
    {#if toolbar}
      <ToolbarButton {title} {icon} onclick={callback} />
    {/if}
  {/each}
</div>

<CommandPalette
  bind:open={commandPaletteOpen}
  bind:this={commandPalette}
  {actions}
/>

<div class="mb-12 p-2 sm:mb-0">
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
        <Row {index} {node} {users} />
      </tbody>
    {/each}
  </table>
</div>
