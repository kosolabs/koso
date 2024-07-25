<script lang="ts">
  import { cn } from "$lib/utils";
  import { A, Input, Tooltip } from "flowbite-svelte";
  import { ChevronRight, Menu } from "lucide-svelte";
  import { getContext } from "svelte";
  import { slide } from "svelte/transition";
  import type { Graph, Node } from ".";
  import { getOffset, getTask } from ".";
  import { selected } from "./state";
  import {
    type IndexedNode,
    type Interactions,
    type TableContext,
  } from "./table.svelte";

  export let graph: Graph;
  export let node: Node;
  export let interactions: Interactions;
  export let isGhost: boolean;

  $: task = getTask(graph, node.name);
  $: ({ dragged, ghost, dropEffect, highlighted } = interactions);

  const {
    koso,
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

  let editedTaskName: string | null = null;

  function handleStartEditingTaskName() {
    editedTaskName = task.name;
  }

  function saveEditedTaskName() {
    if (editedTaskName === null) {
      return;
    }
    koso.editTaskName(node.name, editedTaskName);
    editedTaskName = null;
  }

  function revertEditedTaskName() {
    if (editedTaskName === null) {
      return;
    }
    editedTaskName = null;
  }

  function handleEditedTaskNameBlur() {
    saveEditedTaskName();
  }

  function handleEditedTaskNameKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      revertEditedTaskName();
    }
    if (event.key === "Enter") {
      event.preventDefault();
      saveEditedTaskName();
    }
  }

  function handleDragStart(event: DragEvent) {
    const dataTransfer = event.dataTransfer;
    if (dataTransfer === null) {
      return;
    }
    clearHighlighted();
    setDragged(node, getOffset(graph, node));
    dataTransfer.setData("text/plain", node.id);
    dataTransfer.effectAllowed = "linkMove";
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
      koso.moveNode(
        dragged.node.name,
        dragged.node.parent().name,
        dragged.offset,
        ghost.node.parent().name,
        ghost.offset,
      );
    } else {
      koso.addNode(dragged.node.name, ghost.node.parent().name, ghost.offset);
    }
    clearDragged();
    clearGhost();
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

    setGhost(
      node.parent().concat(dragged.node.name),
      getOffset(graph, node) + 1,
    );
  }

  function handleDragEnterPeer(event: DragEvent) {
    event.preventDefault();
    open = false;
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

  function handleDragEnterChild(event: DragEvent) {
    event.preventDefault();
    open = true;
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

  function handleSelect() {
    $selected = node;
  }

  function hasCycle(parent: string, child: string): boolean {
    if (child === parent) {
      return true;
    }
    for (const next of getTask(graph, child).children) {
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
    return getTask(graph, parent.name).children.includes(child.name);
  }

  function isSamePeer(node: Node, dragged: IndexedNode): boolean {
    if (dragged.node.isRoot()) {
      return false;
    }
    if (!node.parent().equals(dragged.node.parent())) {
      return false;
    }
    return getOffset(graph, node) + 1 === dragged.offset;
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
  $: isSelected = node.equals($selected);
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div
  role="row"
  tabindex="0"
  class={cn(
    "flex items-center border border-transparent p-2",
    isMoving ? "border-red-600 opacity-30" : "",
    isGhost ? "border-green-600 opacity-70" : "",
    highlighted?.name === node.name ? "border-lime-600" : "",
    isSelected ? "border-primary-600" : "",
  )}
  on:mouseover={handleHighlight}
  on:mouseout={handleUnhighlight}
  on:focus={handleHighlight}
  on:blur={handleUnhighlight}
  on:click={handleSelect}
  transition:slide|global={{ duration: interactions.dragged ? 0 : 400 }}
>
  <div class="min-w-48 overflow-x-clip whitespace-nowrap">
    <div class="flex items-center">
      <button
        class="min-w-5 transition-transform"
        class:rotate-90={open && !isGhost}
        style="margin-left: {(node.length - 1) * 1.25}rem;"
        on:click={() => toggleOpen()}
      >
        {#if task.children.length > 0}
          <ChevronRight class="h-4" />
        {/if}
      </button>
      <Tooltip class="text-nowrap" placement="top">
        {open ? "Collapse" : "Expand"}
      </Tooltip>
      <button
        class="relative min-w-5"
        draggable={true}
        on:dragstart={handleDragStart}
        on:dragend={handleDragEnd}
        on:drag={handleDrag}
      >
        <Menu class="h-4" />
        {#if canDragDropPeer}
          <div
            class="absolute z-50 h-7"
            style="width: {(node.length + 1) * 1.25}rem; 
              left: {-node.length * 1.25}rem"
            role="table"
            on:dragover={handleDragOverPeer}
            on:dragenter={handleDragEnterPeer}
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
            on:dragenter={handleDragEnterChild}
            on:dragleave={handleDragLeave}
            on:drop={handleDropNode}
          />
        {/if}
      </button>
      <div class="overflow-x-hidden whitespace-nowrap">{node.name}</div>
    </div>
  </div>
  <div class="w-96 overflow-x-hidden whitespace-nowrap px-2">
    {#if editedTaskName !== null}
      <Input
        size="sm"
        on:blur={handleEditedTaskNameBlur}
        on:keydown={handleEditedTaskNameKeydown}
        bind:value={editedTaskName}
        autofocus
      />
    {:else}
      <A
        class="hover:no-underline"
        on:click={handleStartEditingTaskName}
        on:keydown={handleStartEditingTaskName}
      >
        {task.name}
      </A>
    {/if}
  </div>
  <div class="w-96 overflow-x-hidden whitespace-nowrap px-2">
    {task.reporter}
  </div>
  <div class="w-96 overflow-x-hidden whitespace-nowrap px-2">
    {task.assignee ?? "Unassigned"}
  </div>
</div>

<!-- Ghost is the first child of node. -->
{#if dragged && ghost && !isGhost && node.equals(ghost.node.parent()) && ghost.offset === 0}
  <svelte:self {graph} {interactions} isGhost={true} node={ghost.node} />
{/if}

{#if open && !isGhost}
  {#each task.children as childId}
    <svelte:self
      {graph}
      {interactions}
      isGhost={false}
      node={node.concat(childId)}
    />
  {/each}
{/if}

<!-- Ghost is the peer immedicately proceeding node. -->
{#if dragged && ghost && !isGhost && !node.isRoot() && node
    .parent()
    .equals(ghost.node.parent()) && ghost.offset === getOffset(graph, node) + 1}
  <svelte:self {graph} {interactions} isGhost={true} node={ghost.node} />
{/if}
