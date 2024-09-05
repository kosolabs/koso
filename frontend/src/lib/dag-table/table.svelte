<script lang="ts">
  import { user, type User } from "$lib/auth";
  import { Button } from "$lib/components/ui/button";
  import { Node, type Koso } from "$lib/koso";
  import {
    ListPlus,
    ListTree,
    SquarePen,
    Trash,
    Unlink,
    UserRoundPlus,
  } from "lucide-svelte";
  import { setContext } from "svelte";
  import { flip } from "svelte/animate";
  import Row from "./row.svelte";

  export let koso: Koso;
  export let users: User[];

  const rows: { [key: string]: HTMLDivElement } = {};
  const { nodesAndIds, parents, selectedId } = koso;

  document.onkeydown = (event: KeyboardEvent) => {
    if (
      event.key === "ArrowDown" &&
      !event.altKey &&
      !event.ctrlKey &&
      !event.metaKey &&
      !event.shiftKey
    ) {
      if (koso.nodeIds.length > 0) {
        const selectedIndex = $selectedId
          ? koso.nodeIds.indexOf($selectedId)
          : -1;
        if (selectedIndex === -1) {
          $selectedId = koso.getNode(1).id;
        } else {
          $selectedId = koso.getNode(
            Math.min(selectedIndex + 1, koso.nodeIds.length - 1),
          ).id;
        }
      }
      if ($selectedId !== null) {
        rows[$selectedId].focus();
      }
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (
      event.key === "ArrowUp" &&
      !event.altKey &&
      !event.ctrlKey &&
      !event.metaKey &&
      !event.shiftKey
    ) {
      if (koso.nodeIds.length > 0) {
        const selectedIndex = $selectedId
          ? koso.nodeIds.indexOf($selectedId)
          : -1;
        if (selectedIndex === -1) {
          $selectedId = koso.getNode(koso.nodeIds.length - 1).id;
        } else {
          $selectedId = koso.getNode(Math.max(selectedIndex - 1, 1)).id;
        }
      }
      if ($selectedId !== null) {
        rows[$selectedId].focus();
      }
      event.preventDefault();
      event.stopPropagation();
      return;
    }
  };

  function addRoot() {
    if (!$user) throw new Error("Unauthenticated");
    koso.insertNode(koso.getNode("root"), 0, "Untitled", $user);
  }

  function addPeer() {
    if (!$selectedId) return;
    if (!$user) throw new Error("Unauthenticated");
    const selected = koso.getNode($selectedId);
    $selectedId = koso.insertNode(
      selected.parent(),
      selected.offset + 1,
      "Untitled",
      $user,
    );
  }

  function addChild() {
    if (!$selectedId) return;
    if (!$user) throw new Error("Unauthenticated");
    const selected = koso.getNode($selectedId);
    $selectedId = koso.insertNode(selected, 0, "Untitled", $user);
  }

  function unlink() {
    if (!$selectedId) return;
    const selected = koso.getNode($selectedId);
    koso.unlinkNode(selected);
    $selectedId = null;
  }

  function remove() {
    if (!$selectedId) return;
    const selected = koso.getNode($selectedId);
    koso.deleteNode(selected);
    $selectedId = null;
  }

  setContext<Koso>("koso", koso);
</script>

<div class="sticky top-0 z-30 flex gap-2 px-4 py-2 pb-2 backdrop-blur">
  {#if $selectedId}
    <Button class="text-xs" on:click={addPeer}>
      <ListPlus class="me-2 w-4" />
      Add Task
    </Button>
    <Button class="text-xs" on:click={addChild}>
      <ListTree class="me-2 w-4" />
      Add Child
    </Button>
    {#if $parents[Node.name(Node.parse($selectedId))].length === 1}
      <Button class="text-xs" on:click={remove}>
        <Trash class="me-2 w-4" />
        Delete
      </Button>
    {:else}
      <Button class="text-xs" on:click={unlink}>
        <Unlink class="me-2 w-4" />
        Unlink
      </Button>
    {/if}
  {:else}
    <Button class="text-xs" on:click={addRoot}>
      <ListPlus class="me-2 w-4" />
      Add Task
    </Button>
  {/if}
</div>

<div class="mx-4 mb-4">
  <table class="w-full border-separate border-spacing-0 rounded-md border">
    <thead class="text-left text-xs font-bold uppercase">
      <tr>
        <th class="w-32 p-2">ID</th>
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

    {#each $nodesAndIds[0].slice(1) as nodeId, index (nodeId)}
      {@const node = koso.getNode(nodeId)}
      <tbody animate:flip={{ duration: 250 }}>
        <Row {index} {node} {users} row={(el) => (rows[nodeId] = el)} />
      </tbody>
    {/each}
  </table>
</div>
