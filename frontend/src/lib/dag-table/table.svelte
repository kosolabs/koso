<script lang="ts">
  import { user, type User } from "$lib/auth";
  import { Button } from "$lib/components/ui/button";
  import { KeyBinding } from "$lib/key-binding";
  import { type Koso } from "$lib/koso";
  import { cn } from "$lib/utils";
  import {
    IndentDecrease,
    IndentIncrease,
    ListPlus,
    ListTree,
    MoveDown,
    MoveUp,
    SquarePen,
    Trash,
    UserRoundPlus,
    Undo,
    Redo,
  } from "lucide-svelte";
  import { setContext } from "svelte";
  import { flip } from "svelte/animate";
  import Row from "./row.svelte";

  export let koso: Koso;
  export let users: User[];

  const rows: { [key: string]: HTMLDivElement } = {};
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
      if ($selected !== null) {
        rows[$selected.id].focus();
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
      if ($selected !== null) {
        rows[$selected.id].focus();
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
    koso.insertNode(koso.root, 0, "Untitled", $user);
  }

  function addPeer() {
    if (!$selected) return;
    if (!$user) throw new Error("Unauthenticated");
    $selected = koso.insertNode(
      $selected.parent,
      koso.getOffset($selected) + 1,
      "Untitled",
      $user,
    );
  }

  function addChild() {
    if (!$selected) return;
    if (!$user) throw new Error("Unauthenticated");
    $selected = koso.insertNode($selected, 0, "Untitled", $user);
    koso.expand($selected);
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

<div class="px-4 pt-4 max-md:px-2 max-md:pt-2">
  <table class="w-full border-separate border-spacing-0 rounded-md border">
    <thead class="text-left text-xs font-bold uppercase">
      <tr>
        <th class="w-32 p-2">ID</th>
        {#if $debug}
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
        <th class="border-l p-2 max-md:hidden">Reporter</th>
      </tr>
    </thead>

    {#each [...$nodes].slice(1) as node, index (node.id)}
      <tbody animate:flip={{ duration: 250 }}>
        <Row {index} {node} {users} row={(el) => (rows[node.id] = el)} />
      </tbody>
    {/each}
  </table>
</div>

<div
  class={cn(
    "sticky bottom-0 z-10 flex flex-wrap gap-2 backdrop-blur-sm",
    "px-4 py-2 max-md:px-2",
  )}
>
  {#if $selected}
    <Button class="text-xs" on:click={addPeer}>
      <ListPlus class="me-2 w-4" />
      Add Task
    </Button>
    <Button class="text-xs" on:click={addChild}>
      <ListTree class="me-2 w-4" />
      Add Child
    </Button>
    <Button class="text-xs" on:click={remove}>
      <Trash class="me-2 w-4" />
      Delete
    </Button>
    <Button class="text-xs" on:click={moveUp}>
      <MoveUp class="w-4" />
    </Button>
    <Button class="text-xs" on:click={moveDown}>
      <MoveDown class="w-4" />
    </Button>
    <Button class="text-xs" on:click={undent}>
      <IndentDecrease class="w-4" />
    </Button>
    <Button class="text-xs" on:click={indent}>
      <IndentIncrease class="w-4" />
    </Button>
  {:else}
    <Button class="text-xs" on:click={addRoot}>
      <ListPlus class="me-2 w-4" />
      Add Task
    </Button>
  {/if}
  <Button class="text-xs" on:click={undo}>
    <Undo class="w-4" />
  </Button>
  <Button class="text-xs" on:click={redo}>
    <Redo class="w-4" />
  </Button>
</div>
