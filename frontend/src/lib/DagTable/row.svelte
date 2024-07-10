<script lang="ts">
  import { A, Input } from "flowbite-svelte";
  import { AngleRightOutline, BarsOutline } from "flowbite-svelte-icons";
  import { getContext } from "svelte";
  import { slide } from "svelte/transition";
  import type { Graph, Node } from ".";
  import { type Interactions, type TableContext } from "./table.svelte";

  export let graph: Graph;
  export let node: Node;
  export let interactions: Interactions;

  $: task = graph[node.name]!;
  $: ghost = interactions.ghost;
  $: isGhost = ghost && node.equals(ghost.node);

  const {
    addNode,
    moveNode,
    setDragged,
    editTaskName,
    clearDragged,
    setGhost,
    clearGhost,
    setHighlighted,
    clearHighlighted,
  } = getContext<TableContext>("graph");

  let open = true;
  function toggleOpen() {
    open = !open;
  }

  let editedDescription: string | null = null;

  function handleStartEditing() {
    editedDescription = task.name;
  }

  function handleEndEditing() {
    if (editedDescription === null) {
      return;
    }
    editTaskName(node.name, editedDescription);
    editedDescription = null;
  }

  function handleDragStart(event: DragEvent) {
    setDragged(node);
    event.dataTransfer!.setData("text/plain", node.id);
    event.dataTransfer!.effectAllowed = "linkMove";

    const rowEl = document.getElementById(`row-${node.id}`)!;
    const handleEl = document.getElementById(`handle-${node.id}`)!;
    const rowRect = rowEl.getBoundingClientRect();
    const handleRect = handleEl.getBoundingClientRect();

    event.dataTransfer!.setDragImage(
      rowEl,
      handleRect.x - rowRect.x + event.offsetX,
      handleRect.y - rowRect.y + event.offsetY,
    );
  }

  function handleDragEnd() {
    clearDragged();
  }

  function handleDropPeer(event: DragEvent) {
    if (interactions.dragged === null) {
      return;
    }

    const nodeId = interactions.dragged.name;
    const sourceId = interactions.dragged.parent().name;
    const parentId = node.parent().name;
    const offset = graph[parentId]!.children.indexOf(node.name) + 1;

    if (event.dataTransfer?.effectAllowed !== "link") {
      moveNode(nodeId, sourceId, parentId, offset);
    } else {
      addNode(nodeId, parentId, offset);
    }
    clearDragged();
    clearGhost();
  }

  function handleDropChild(event: DragEvent) {
    if (interactions.dragged === null) {
      return;
    }

    const nodeId = interactions.dragged.name;
    const sourceId = interactions.dragged.parent().name;
    const parentId = node.name;
    const offset = 0;

    if (event.dataTransfer?.effectAllowed !== "link") {
      moveNode(nodeId, sourceId, parentId, offset);
    } else {
      addNode(nodeId, parentId, offset);
    }
    clearDragged();
    clearGhost();
  }

  function handleDragOverPeer() {
    if (interactions.dragged === null) {
      return;
    }

    const parentId = node.parent().name;
    const offset = graph[parentId]!.children.indexOf(node.name) + 1;
    setGhost(node.parent().concat(interactions.dragged.name), offset);
  }

  function handleDragOverChild() {
    if (interactions.dragged === null) {
      return;
    }
    setGhost(node.concat(interactions.dragged.name), 0);
  }

  function handleDragLeave() {
    clearGhost();
  }

  function handleHighlight() {
    setHighlighted(node);
  }

  function handleUnhighlight() {
    clearHighlighted();
  }

  function hasCycle(parent: string, child: string): boolean {
    if (child === parent) {
      return true;
    }
    for (const next of graph[child]!.children) {
      if (hasCycle(parent, next)) {
        return true;
      }
    }
    return false;
  }

  function isValidRelationship(parent: Node, child: Node | null) {
    if (child === null) {
      return false;
    }
    if (parent.length === 0) {
      return false;
    }
    const parentId = parent.name;
    const childId = child.name;
    if (hasCycle(parentId, childId)) {
      return false;
    }
    if (graph[parentId]?.children.includes(childId)) {
      return false;
    }
    return true;
  }

  function children() {
    if (ghost && node.equals(ghost.node.parent())) {
      return task.children.toSpliced(ghost.offset, 0, ghost.node.name);
    }
    return task.children;
  }

  $: canDragDropPeer = isValidRelationship(node.parent(), interactions.dragged);
  $: canDragDropChild = isValidRelationship(node, interactions.dragged);
</script>

<div
  id="row-{node.id}"
  role="row"
  tabindex="0"
  class="my-1 flex items-center rounded border p-2"
  class:border-lime-600={interactions.highlighted?.name === node.name}
  class:opacity-50={isGhost}
  on:mouseover={handleHighlight}
  on:mouseout={handleUnhighlight}
  on:focus={handleHighlight}
  on:blur={handleUnhighlight}
  transition:slide|global={{ duration: isGhost ? 0 : 400 }}
>
  <div class="w-48">
    <div class="flex items-center">
      <button
        class="min-w-5 transition-transform"
        class:rotate-90={open && !isGhost}
        style="margin-left: {(node.length - 1) * 1.25}rem;"
        on:click={() => toggleOpen()}
      >
        {#if task.children.length > 0}
          <AngleRightOutline class="h-4" />
        {/if}
      </button>
      <button
        id="handle-{node.id}"
        class="relative min-w-5"
        draggable={true}
        on:dragstart={(event) => handleDragStart(event)}
        on:dragend|preventDefault={() => handleDragEnd()}
      >
        <BarsOutline class="h-4" />
        {#if canDragDropPeer}
          <div
            id="peer-dropzone-{node.id}"
            class="absolute -left-6 z-50 h-7 w-12 bg-red-400"
            role="table"
            on:dragover|preventDefault={() => handleDragOverPeer()}
            on:dragleave|preventDefault={() => handleDragLeave()}
            on:drop|preventDefault={(event) => handleDropPeer(event)}
          />
        {/if}
        {#if canDragDropChild}
          <div
            id="child-dropzone-{node.id}"
            class="absolute left-6 z-50 h-7 w-12 bg-blue-400"
            role="table"
            on:dragover|preventDefault={() => handleDragOverChild()}
            on:dragleave|preventDefault={() => handleDragLeave()}
            on:drop|preventDefault={(event) => handleDropChild(event)}
          />
        {/if}
      </button>
      <div>{node.name}</div>
    </div>
  </div>
  <div class="w-96 px-2">
    {#if editedDescription !== null}
      <Input
        size="sm"
        on:blur={() => handleEndEditing()}
        bind:value={editedDescription}
        autofocus
      />
    {:else}
      <A
        class="hover:no-underline"
        on:click={() => handleStartEditing()}
        on:keydown={() => handleStartEditing()}
      >
        {task.name}
      </A>
    {/if}
  </div>
</div>

{#if open && !isGhost}
  {#each children() as child}
    <svelte:self {graph} {interactions} node={node.concat(child)} />
  {/each}
{/if}
