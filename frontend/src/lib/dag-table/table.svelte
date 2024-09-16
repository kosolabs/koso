<script lang="ts">
  import { user, type User } from "$lib/auth";
  import { KeyBinding } from "$lib/key-binding";
  import { type Koso } from "$lib/koso";
  import { globalKeybindingsEnabled } from "$lib/popover-monitors";
  import ToolbarButton from "$lib/toolbar-button.svelte";
  import { cn } from "$lib/utils";
  import {
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
  import { flip } from "svelte/animate";
  import Row from "./row.svelte";

  export let koso: Koso;
  export let users: User[];

  const { debug, nodes, selected } = koso;

  function moveUp() {
    if (!$selected) return;
    koso.moveNodeUp($selected);
  }

  function moveDown() {
    if (!$selected) return;
    koso.moveNodeDown($selected);
  }

  function indent() {
    if (!$selected) return;
    koso.indentNode($selected);
  }

  function undent() {
    if (!$selected) return;
    koso.undentNode($selected);
  }

  document.onkeydown = (event: KeyboardEvent) => {
    if (!globalKeybindingsEnabled()) return;

    if (KeyBinding.INDENT_NODE.equals(event)) {
      indent();
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (KeyBinding.UNDENT_NODE.equals(event)) {
      undent();
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (KeyBinding.MOVE_NODE_UP.equals(event)) {
      moveUp();
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (KeyBinding.MOVE_NODE_DOWN.equals(event)) {
      moveDown();
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (KeyBinding.COLLAPSE_NODE.equals(event)) {
      if (!$selected) return;
      koso.collapse($selected);
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (KeyBinding.EXPAND_NODE.equals(event)) {
      if (!$selected) return;
      koso.expand($selected);
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (KeyBinding.SELECT_NEXT_NODE.equals(event)) {
      if ($nodes.size > 1) {
        if ($selected) {
          $nodes.indexOf($selected);
          const index = Math.min(
            $nodes.indexOf($selected) + 1,
            $nodes.size - 1,
          );
          $selected = $nodes.get(index, null);
        } else {
          $selected = $nodes.get(1, null);
        }
      }
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (KeyBinding.SELECT_PREV_NODE.equals(event)) {
      if ($nodes.size > 1) {
        if ($selected) {
          const index = Math.max($nodes.indexOf($selected) - 1, 1);
          $selected = $nodes.get(index, null);
        } else {
          $selected = $nodes.get($nodes.size - 1, null);
        }
      }
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (KeyBinding.UNDO.equals(event)) {
      undo();

      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (KeyBinding.REDO.equals(event)) {
      redo();

      event.preventDefault();
      event.stopPropagation();
      return;
    }
  };

  function addRoot() {
    if (!$user) throw new Error("Unauthenticated");
    koso.insertNode(koso.root, 0, $user);
  }

  function addPeer() {
    if (!$selected) return;
    if (!$user) throw new Error("Unauthenticated");
    koso.insertNode($selected.parent, koso.getOffset($selected) + 1, $user);
  }

  function addChild() {
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

  function undo() {
    koso.undo();
  }

  function redo() {
    koso.redo();
  }

  setContext<Koso>("koso", koso);
</script>

<div
  class={cn(
    "z-10 flex items-center overflow-x-scroll px-2 backdrop-blur-sm",
    "fixed bottom-0 left-0 h-12 w-full border-t",
    "sm:sticky sm:top-0 sm:gap-2 sm:border-b",
  )}
>
  {#if $selected}
    <ToolbarButton title="Add Task" icon={ListPlus} on:click={addPeer} />
    <ToolbarButton title="Add Child" icon={ListTree} on:click={addChild} />
    <ToolbarButton title="Delete" icon={Trash} on:click={remove} />
    <ToolbarButton title="Move Up" icon={MoveUp} on:click={moveUp} />
    <ToolbarButton title="Move Down" icon={MoveDown} on:click={moveDown} />
    <ToolbarButton title="Undent" icon={IndentDecrease} on:click={undent} />
    <ToolbarButton title="Indent" icon={IndentIncrease} on:click={indent} />
  {:else}
    <ToolbarButton title="Add Task" icon={ListPlus} on:click={addRoot} />
  {/if}
  <ToolbarButton title="Undo" icon={Undo} on:click={undo} />
  <ToolbarButton title="Redo" icon={Redo} on:click={redo} />
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
