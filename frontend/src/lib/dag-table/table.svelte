<script lang="ts">
  import { user, type User } from "$lib/auth";
  import { Button } from "$lib/components/ui/button";
  import { CommandPalette } from "$lib/components/ui/command-palette";
  import { ShortcutChips } from "$lib/components/ui/shortcut";
  import { ToolbarButton } from "$lib/components/ui/toolbar-button";
  import { type Koso } from "$lib/koso";
  import { globalKeybindingsEnabled } from "$lib/popover-monitors";
  import { Shortcut, ShortcutRegistry, type Action } from "$lib/shortcuts";
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

  type Props = {
    koso: Koso;
    users: User[];
  };
  const { koso, users }: Props = $props();
  const { debug, editing, nodes, selected, showDone } = koso;

  let commandPaletteOpen: boolean = $state(false);
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

  const actions: Action[] = [
    {
      title: "Add Task",
      icon: ListPlus,
      callback: insert,
      toolbar: true,
      enabled: () => true,
      shortcut: Shortcut.INSERT_NODE,
    },
    {
      title: "Edit Name",
      icon: Pencil,
      callback: edit,
      toolbar: false,
      enabled: () => true,
      shortcut: Shortcut.EDIT_NODE,
    },
    {
      title: "Cancel Selection",
      icon: CircleX,
      callback: unselect,
      toolbar: false,
      enabled: () => true,
      shortcut: Shortcut.CANCEL_SELECTION,
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
      toolbar: false,
      enabled: () => !!$selected,
      shortcut: Shortcut.MOVE_NODE_ROW_UP,
    },
    {
      title: "Move Row Down",
      icon: MoveDown,
      callback: moveRowDown,
      toolbar: false,
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
      toolbar: false,
      enabled: () => !!$selected,
      shortcut: Shortcut.UNDENT_NODE_SHIFT,
    },
    {
      title: "Indent",
      icon: IndentIncrease,
      callback: indent,
      toolbar: false,
      enabled: () => !!$selected,
      shortcut: Shortcut.INDENT_NODE_SHIFT,
    },
    {
      title: "Undo",
      icon: Undo,
      callback: undo,
      toolbar: true,
      enabled: () => true,
      shortcut: Shortcut.UNDO,
    },
    {
      title: "Redo",
      icon: Redo,
      callback: redo,
      toolbar: true,
      enabled: () => true,
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
      toolbar: false,
      enabled: () => true,
      shortcut: Shortcut.EXPAND_NODE,
    },
    {
      title: "Collapse Task",
      icon: ChevronsDownUp,
      callback: collapse,
      toolbar: false,
      enabled: () => true,
      shortcut: Shortcut.COLLAPSE_NODE,
    },
    {
      title: "Select Next Task",
      icon: StepForward,
      callback: selectNext,
      toolbar: false,
      enabled: () => true,
      shortcut: Shortcut.SELECT_NEXT_NODE,
    },
    {
      title: "Select Previous Task",
      icon: StepBack,
      callback: selectPrev,
      toolbar: false,
      enabled: () => true,
      shortcut: Shortcut.SELECT_PREV_NODE,
    },
    {
      title: "Show Command Palette",
      icon: Terminal,
      callback: showCommandPalette,
      toolbar: true,
      enabled: () => true,
      shortcut: Shortcut.SHOW_COMMAND_PALETTE,
    },
  ];
  const registry = new ShortcutRegistry(actions);

  const toolbarActions = $derived(
    actions.filter((action) => action.toolbar && action.enabled()),
  );

  document.onkeydown = (event: KeyboardEvent) => {
    if ($debug) {
      // TODO: Remove any once toast support Component type
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      toast.info(ShortcutChips as any, {
        componentProps: { binding: Shortcut.fromEvent(event) },
      });
    }

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
  {#each toolbarActions as action}
    <ToolbarButton {...action} />
  {/each}
</div>

<CommandPalette bind:open={commandPaletteOpen} {actions} />

<div class="mb-12 p-2 sm:mb-0">
  {#if $nodes.size > 1}
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
  {:else}
    <div class="flex items-center justify-center">
      Welcome to Koso! Add your first task
      <Button size="sm" onclick={insert} class="ml-2">
        <ListPlus class="w-4 sm:me-2" />
        Add task
      </Button>
    </div>
  {/if}
</div>
