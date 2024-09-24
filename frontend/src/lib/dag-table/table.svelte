<script lang="ts">
  import { user, type User } from "$lib/auth";
  import { KeyBinding } from "$lib/key-binding";
  import { KeyHandlerRegistry } from "$lib/key-handler-registry";
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
  import { toast } from "svelte-sonner";
  import { flip } from "svelte/animate";
  import Row from "./row.svelte";

  export let koso: Koso;
  export let users: User[];

  const { debug, nodes, selected } = koso;

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

  function expand() {
    if (!$selected) return;
    koso.expand($selected);
  }

  function collapse() {
    if (!$selected) return;
    koso.collapse($selected);
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
    [KeyBinding.MOVE_NODE_UP, moveUp],
    [KeyBinding.MOVE_NODE_DOWN, moveDown],
    [KeyBinding.INDENT_NODE, indent],
    [KeyBinding.UNDENT_NODE, undent],
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
    "fixed bottom-0 left-0 z-10 flex h-12 w-full items-center overflow-x-scroll border-t px-2 backdrop-blur-sm sm:sticky sm:top-0 sm:gap-2 sm:border-b",
  )}
>
  <ToolbarButton Icon={ListPlus} title="Add Task" onclick={insert} />
  {#if $selected}
    <ToolbarButton Icon={ListTree} title="Add Child" onclick={insertChild} />
    <ToolbarButton Icon={Trash} title="Delete" onclick={remove} />
    <ToolbarButton Icon={MoveUp} title="Move Up" onclick={moveUp} />
    <ToolbarButton Icon={MoveDown} title="Move Down" onclick={moveDown} />
    <ToolbarButton Icon={IndentDecrease} title="Undent" onclick={undent} />
    <ToolbarButton Icon={IndentIncrease} title="Indent" onclick={indent} />
  {/if}
  <ToolbarButton Icon={Undo} title="Undo" onclick={undo} />
  <ToolbarButton Icon={Redo} title="Redo" onclick={redo} />
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
