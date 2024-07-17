<script lang="ts">
  import { A, Input } from "flowbite-svelte";
  import {
    AngleRightOutline,
    BarsOutline,
    TrashBinOutline,
  } from "flowbite-svelte-icons";
  import { getContext } from "svelte";
  import { slide } from "svelte/transition";
  import type { Graph, Node } from ".";
  import {
    type IndexedNode,
    type Interactions,
    type TableContext,
  } from "./table.svelte";

  export let graph: Graph;
  export let node: Node;
  export let interactions: Interactions;
  export let isGhost: boolean;
  export let offset: number;

  $: task = graph[node.name]!;
  $: ({ dragged, ghost, dropEffect, highlighted } = interactions);

  const {
    kosoGraph,
    setDragged,
    clearDragged,
    setDropEffect,
    setGhost,
    clearGhost,
    setHighlighted,
    clearHighlighted,
  } = getContext<TableContext>("graph");

  let open = true;
  function toggleOpen() {
    open = !open;
  }

  let unlinking = false;

  let editedDescription: string | null = null;

  function handleStartEditing() {
    editedDescription = task.name;
  }

  function handleEndEditing() {
    if (editedDescription === null) {
      return;
    }
    kosoGraph.editTaskName(node.name, editedDescription);
    editedDescription = null;
  }

  function handleDropUnlink(event: DragEvent) {
    event.preventDefault();
    kosoGraph.removeNode(node.name, node.parent().name);
  }

  function handleDragStart(event: DragEvent) {
    clearHighlighted();
    setDragged(
      node,
      node.isRoot()
        ? 0
        : graph[node.parent().name]!.children.indexOf(node.name),
    );
    event.dataTransfer!.setData("text/plain", node.id);
    event.dataTransfer!.effectAllowed = "linkMove";
  }

  function handleDrag(event: DragEvent) {
    event.preventDefault();
  }

  function handleDragEnd(event: DragEvent) {
    event.preventDefault();
    clearDragged();
  }

  function handleDropNode(event: DragEvent) {
    event.preventDefault();
    if (dragged === null || ghost === null || dropEffect === "none") {
      return;
    }

    if (!dragged.node.isRoot() && dropEffect === "move") {
      kosoGraph.moveNode(
        dragged.node.name,
        dragged.node.parent().name,
        dragged.offset,
        ghost.node.parent().name,
        ghost.offset,
      );
    } else {
      kosoGraph.addNode(
        dragged.node.name,
        ghost.node.parent().name,
        ghost.offset,
      );
    }
    clearDragged();
    clearGhost();
  }

  function handleDragOverUnlink(event: DragEvent) {
    event.preventDefault();
    unlinking = true;
  }

  function handleDragLeaveUnlink(event: DragEvent) {
    event.preventDefault();
    unlinking = false;
  }

  function handleDragOverPeer(event: DragEvent) {
    event.preventDefault();
    const dataTransfer = event.dataTransfer;
    if (dragged === null || dataTransfer === null) {
      return;
    }

    if (dragged.node.isRoot() || dragged.node.parent().equals(node.parent())) {
      dataTransfer.dropEffect = "move";
      setDropEffect("move");
    } else {
      setDropEffect(dataTransfer.effectAllowed === "link" ? "link" : "move");
    }

    const parentId = node.parent().name;
    const offset = graph[parentId]!.children.indexOf(node.name) + 1;
    setGhost(node.parent().concat(dragged.node.name), offset);
  }

  function handleDragOverChild(event: DragEvent) {
    event.preventDefault();
    const dataTransfer = event.dataTransfer;
    if (dragged === null || dataTransfer === null) {
      return;
    }

    if (dragged.node.isRoot() || dragged.node.parent().equals(node)) {
      dataTransfer.dropEffect = "move";
      setDropEffect("move");
    } else {
      setDropEffect(dataTransfer.effectAllowed === "link" ? "link" : "move");
    }

    setGhost(node.concat(dragged.node.name), 0);
  }

  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    clearGhost();
  }

  function handleHighlight() {
    if (dragged) {
      return;
    }
    setHighlighted(node);
  }

  function handleUnhighlight() {
    if (dragged) {
      return;
    }
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

  function hasChild(parent: Node, child: Node): boolean {
    if (child.isRoot()) {
      return false;
    }
    if (parent.equals(child.parent())) {
      return false;
    }
    return graph[parent.name]?.children.includes(child.name);
  }

  function isSamePeer(node: Node, dragged: IndexedNode): boolean {
    if (dragged.node.isRoot()) {
      return false;
    }
    if (!node.parent().equals(dragged.node.parent())) {
      return false;
    }

    const parentId = node.parent().name;
    const children = graph[parentId]!.children;
    return children.indexOf(node.name) + 1 === dragged.offset;
  }

  function isSameChild(node: Node, dragged: IndexedNode): boolean {
    if (dragged.node.isRoot()) {
      return false;
    }
    if (!node.equals(dragged.node.parent())) {
      return false;
    }
    return dragged.offset === 0;
  }

  $: dragging = !isGhost && dragged && node.equals(dragged.node);
  $: canDragDropPeer =
    !dragging &&
    !node.isRoot() &&
    dragged &&
    !isSamePeer(node, dragged) &&
    !hasChild(node.parent(), dragged.node) &&
    !hasCycle(node.parent().name, dragged.node.name);
  $: canDragDropChild =
    !dragging &&
    dragged &&
    !isSameChild(node, dragged) &&
    !hasChild(node, dragged.node) &&
    !hasCycle(node.name, dragged.node.name);
  $: isMoving = ghost && dragging && dropEffect === "move";
</script>

<div
  role="row"
  tabindex="0"
  class="my-1 flex items-center rounded border p-2
    {isMoving || unlinking ? 'border-red-600 opacity-30' : ''}
    {isGhost ? 'border-green-600 opacity-70' : ''}
    {highlighted?.name === node.name ? 'border-lime-600' : ''}"
  on:mouseover={handleHighlight}
  on:mouseout={handleUnhighlight}
  on:focus={handleHighlight}
  on:blur={handleUnhighlight}
  transition:slide|global={{ duration: interactions.dragged ? 0 : 400 }}
>
  {#if dragging}
    <div
      class="absolute left-1 z-50 rounded p-1 opacity-50 outline hover:opacity-100"
      role="table"
      on:dragover={handleDragOverUnlink}
      on:dragleave={handleDragLeaveUnlink}
      on:drop={handleDropUnlink}
    >
      <TrashBinOutline />
    </div>
  {/if}
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
        class="relative min-w-5"
        draggable={true}
        on:dragstart={handleDragStart}
        on:dragend={handleDragEnd}
        on:drag={handleDrag}
      >
        <BarsOutline class="h-4" />
        {#if canDragDropPeer}
          <div
            class="absolute z-50 h-7"
            style="width: {(node.length + 1) * 1.25}rem; 
              left: {-node.length * 1.25}rem"
            role="table"
            on:dragover={handleDragOverPeer}
            on:dragleave={handleDragLeave}
            on:drop={handleDropNode}
          />
        {/if}
        {#if canDragDropChild}
          <div
            class="absolute left-5 z-50 h-7"
            style="width: {10.5 - node.length * 1.25}rem"
            role="table"
            on:dragover={handleDragOverChild}
            on:dragleave={handleDragLeave}
            on:drop={handleDropNode}
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

<!-- Ghost is the first child of node. -->
{#if dragged && ghost && node.equals(ghost.node.parent()) && ghost.offset === 0}
  <svelte:self
    {graph}
    {interactions}
    isGhost={true}
    node={ghost.node}
    offset={ghost.offset}
  />
{/if}

{#if open && !isGhost}
  {#each task.children as childId, offset}
    <svelte:self
      {graph}
      {interactions}
      isGhost={false}
      node={node.concat(childId)}
      {offset}
    />
  {/each}
{/if}

<!-- Ghost is the peer immedicately proceeding node. -->
{#if dragged && ghost && !node.isRoot() && node
    .parent()
    .equals(ghost.node.parent()) && ghost.offset === offset + 1}
  <svelte:self
    {graph}
    {interactions}
    isGhost={true}
    node={ghost.node}
    offset={ghost.offset}
  />
{/if}
