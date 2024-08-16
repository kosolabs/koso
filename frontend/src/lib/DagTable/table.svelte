<script lang="ts">
  import { user, type User } from "$lib/auth";
  import type { Koso } from "$lib/koso";
  import { Button } from "flowbite-svelte";
  import { List, ListStart, ListTree, Trash, Unlink } from "lucide-svelte";
  import { UserRoundPlus, SquarePen } from "lucide-svelte";
  import { setContext } from "svelte";
  import { flip } from "svelte/animate";
  import Row from "./row.svelte";
  import { graph, nodes, parents, selected } from "./state";

  export let koso: Koso;
  export let users: User[];
  let rows: { [key: string]: HTMLDivElement } = {};

  koso.observe(() => {
    $graph = koso.toJSON();
  });

  document.onkeydown = (event: KeyboardEvent) => {
    if (event.key === "ArrowDown") {
      if ($nodes.length > 0) {
        const paths = $nodes.map((node) => node.id);
        const selectedIndex = $selected ? paths.indexOf($selected.id) : -1;
        if (selectedIndex === -1) {
          $selected = $nodes[0];
        } else {
          $selected = $nodes[Math.min(selectedIndex + 1, paths.length - 1)];
        }
      }
      if ($selected !== null) {
        rows[$selected.id].focus();
      }
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (event.key === "ArrowUp") {
      if ($nodes.length > 0) {
        const paths = $nodes.map((node) => node.id);
        const selectedIndex = $selected ? paths.indexOf($selected.id) : -1;
        if (selectedIndex === -1) {
          $selected = $nodes[$nodes.length - 1];
        } else {
          $selected = $nodes[Math.max(selectedIndex - 1, 0)];
        }
      }
      if ($selected !== null) {
        rows[$selected.id].focus();
      }
      event.preventDefault();
      event.stopPropagation();
      return;
    }
  };

  function addRoot() {
    if (!$user) throw new Error("Unauthenticated");
    koso.insertNode("root", 0, "Untitled", $user);
  }

  function addPeer() {
    if (!$selected) return;
    if (!$user) throw new Error("Unauthenticated");
    const parent = $selected.parent();
    const newNodeId = koso.insertNode(
      parent.name,
      koso.getOffset($selected) + 1,
      "Untitled",
      $user,
    );
    $selected = parent.concat(newNodeId);
  }

  function addChild() {
    if (!$selected) return;
    if (!$user) throw new Error("Unauthenticated");
    const newNodeId = koso.insertNode($selected.name, 0, "Untitled", $user);
    $selected = $selected.concat(newNodeId);
  }

  function unlink() {
    if (!$selected) return;
    koso.unlinkNode($selected);
    $selected = null;
  }

  function remove() {
    if (!$selected) return;
    koso.deleteNode($selected);
    $selected = null;
  }

  setContext<Koso>("koso", koso);
</script>

<div class="sticky top-0 z-30 flex gap-2 bg-white py-2">
  {#if $selected}
    <Button size="xs" on:click={addPeer}>
      <List class="me-2 w-4" />Add Peer
    </Button>
    <Button size="xs" on:click={addChild}>
      <ListTree class="me-2 w-4" />Add Child
    </Button>
    {#if $parents[$selected.name].length === 1}
      <Button size="xs" on:click={remove}>
        <Trash class="me-2 w-4" />Delete
      </Button>
    {:else}
      <Button size="xs" on:click={unlink}>
        <Unlink class="me-2 w-4" />Unlink
      </Button>
    {/if}
  {:else}
    <Button size="xs" on:click={addRoot}>
      <ListStart class="me-2 w-4" />Add Root
    </Button>
  {/if}
</div>

<table class="w-full border-separate border-spacing-0 border-b border-l">
  <thead
    class="sticky top-14 z-10 bg-white text-left text-xs font-bold uppercase"
  >
    <tr>
      <th class="w-32 border-r border-t p-2">ID</th>
      <th class="border-r border-t p-2">
        <SquarePen class="h-4 md:hidden" />
        <div class="max-md:hidden">Status</div></th
      >
      <th class="border-r border-t p-2">Name</th>
      <th class="border-r border-t p-2">
        <UserRoundPlus class="h-4 md:hidden" />
        <div class="max-md:hidden">Assignee</div>
      </th>
      <th class="border-r border-t p-2 max-md:hidden">Reporter</th>
    </tr>
  </thead>

  {#each $nodes as node, index (node.id)}
    <tbody animate:flip={{ duration: 250 }}>
      <Row
        {index}
        {node}
        {users}
        isGhost={false}
        rowCallback={(el) => (rows[node.id] = el)}
      />
    </tbody>
  {/each}
</table>
