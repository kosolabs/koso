<script lang="ts">
  import { user, type User } from "$lib/auth";
  import { KeyBinding } from "$lib/key-binding";
  import { KeyHandlerRegistry } from "$lib/key-handler-registry";
  import { type Koso } from "$lib/koso";
  import { globalKeybindingsEnabled } from "$lib/popover-monitors";
  import ToolbarButton from "$lib/toolbar-button.svelte";
  import { cn } from "$lib/utils";
  import {
    Eye,
    EyeOff,
    IndentDecrease,
    IndentIncrease,
    ListPlus,
    ListTree,
    MoveDown,
    MoveUp,
    Redo,
    SquarePen,
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

  const { debug, editing, nodes, selected, showDone } = koso;

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
    koso.deleteNode($selected);
    $selected = null;
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

  const registry = new KeyHandlerRegistry([
    [KeyBinding.INSERT_NODE, insert],
    [KeyBinding.REMOVE_NODE, remove],
    [KeyBinding.EDIT_NODE, edit],
    [KeyBinding.CANCEL_SELECTION, unselect],
    [KeyBinding.INSERT_CHILD_NODE, insertChild],
    [KeyBinding.MOVE_NODE_UP, moveUp],
    [KeyBinding.MOVE_NODE_DOWN, moveDown],
    [KeyBinding.MOVE_NODE_ROW_UP, moveRowUp],
    [KeyBinding.MOVE_NODE_ROW_DOWN, moveRowDown],
    [KeyBinding.INDENT_NODE, indent],
    [KeyBinding.INDENT_NODE_SHIFT, indent],
    [KeyBinding.UNDENT_NODE, undent],
    [KeyBinding.UNDENT_NODE_SHIFT, undent],
    [KeyBinding.EXPAND_NODE, expand],
    [KeyBinding.COLLAPSE_NODE, collapse],
    [KeyBinding.SELECT_NEXT_NODE, selectNext],
    [KeyBinding.SELECT_PREV_NODE, selectPrev],
    [KeyBinding.UNDO, undo],
    [KeyBinding.REDO, redo],
  ]);

  document.onkeydown = (event: KeyboardEvent) => {
    if ($debug) {
      toast.info(JSON.stringify(KeyBinding.fromEvent(event).toJSON()));
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
  <ToolbarButton title="Add Task" icon={ListPlus} on:click={insert} />
  {#if $selected}
    <ToolbarButton title="Add Child" icon={ListTree} on:click={insertChild} />
    <ToolbarButton title="Delete" icon={Trash} on:click={remove} />
    <ToolbarButton title="Move Up" icon={MoveUp} on:click={moveUp} />
    <ToolbarButton title="Move Down" icon={MoveDown} on:click={moveDown} />
    <ToolbarButton title="Undent" icon={IndentDecrease} on:click={undent} />
    <ToolbarButton title="Indent" icon={IndentIncrease} on:click={indent} />
  {/if}
  <ToolbarButton title="Undo" icon={Undo} on:click={undo} />
  <ToolbarButton title="Redo" icon={Redo} on:click={redo} />
  {#if $showDone}
    <ToolbarButton
      title="Hide Done Tasks"
      icon={EyeOff}
      on:click={hideDoneTasks}
    />
  {:else}
    <ToolbarButton
      title="Show Done Tasks"
      icon={Eye}
      on:click={showDoneTasks}
    />
  {/if}
</div>

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
