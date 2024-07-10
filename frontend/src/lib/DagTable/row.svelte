<script lang="ts">
  import { A, Input } from "flowbite-svelte";
  import { AngleRightOutline, BarsOutline } from "flowbite-svelte-icons";
  import { getContext } from "svelte";
  import type { Graph, Path } from ".";
  import { type Interactions, type TableContext } from "./table.svelte";

  export let graph: Graph;
  export let path: Path;
  export let interactions: Interactions;
  export let ghost = false;

  $: node = graph[path.name]!;

  const {
    addNode,
    moveNode,
    setDragged,
    editTaskName,
    clearDragged,
    setMaybePeer,
    setMaybeChild,
    clearMaybePeerAndChild,
    setHighlighted,
    clearHighlighted,
  } = getContext<TableContext>("graph");

  let open = true;
  function toggleOpen() {
    open = !open;
  }

  let editedDescription: string | null = null;

  function startEditing() {
    editedDescription = node.name;
  }
  function stopEditing() {
    if (editedDescription === null) {
      return;
    }
    editTaskName(path.name, editedDescription);
    editedDescription = null;
  }

  function dragStart(event: DragEvent) {
    setDragged(path);
    event.dataTransfer!.setData("text/plain", path.id);
    event.dataTransfer!.effectAllowed = "linkMove";

    const rowEl = document.getElementById(`row-${path.id}`)!;
    const handleEl = document.getElementById(`handle-${path.id}`)!;
    const rowRect = rowEl.getBoundingClientRect();
    const handleRect = handleEl.getBoundingClientRect();

    event.dataTransfer!.setDragImage(
      rowEl,
      handleRect.x - rowRect.x + event.offsetX,
      handleRect.y - rowRect.y + event.offsetY,
    );
  }

  function dragEnd(event: DragEvent) {
    event.preventDefault();
    clearDragged();
  }

  function drop(event: DragEvent, relationship: "peer" | "child") {
    event.preventDefault();
    console.log(
      "drop",
      event.dataTransfer!.dropEffect,
      event.dataTransfer!.effectAllowed,
    );

    if (interactions.dragged === null) {
      return;
    }

    let nodeId = interactions.dragged.name;
    let srcParentId = interactions.dragged.parent().name;
    let destParentId =
      relationship === "child" ? path.name : path.parent().name;
    let offset =
      relationship === "child"
        ? 0
        : graph[destParentId]!.children.indexOf(path.name) + 1;

    if (event.dataTransfer?.effectAllowed !== "link") {
      moveNode(nodeId, srcParentId, destParentId, offset);
    } else {
      addNode(nodeId, destParentId, offset);
    }
    clearDragged();
    clearMaybePeerAndChild();
  }

  function dragOver(event: DragEvent, relationship: "peer" | "child") {
    event.preventDefault();
    if (interactions.dragged === null) {
      return;
    }
    if (relationship === "child") {
      setMaybeChild(path);
    } else if (relationship === "peer") {
      setMaybePeer(path);
    }
  }

  function dragLeave(event: DragEvent) {
    event.preventDefault();
    clearMaybePeerAndChild();
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

  function isValidRelationship(parent: Path, child: Path | null) {
    if (child === null) {
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

  function highlight() {
    setHighlighted(path);
  }

  function unhighlight() {
    clearHighlighted();
  }

  $: canDragDrop = isValidRelationship(path, interactions.dragged);
</script>

<div
  id="row-{path.id}"
  class="my-1 flex items-center rounded border p-2"
  class:border-lime-600={interactions.highlighted?.name === path.name}
  class:opacity-50={ghost}
  on:mouseover={highlight}
  on:mouseout={unhighlight}
  on:focus={highlight}
  on:blur={unhighlight}
  role="row"
  tabindex="0"
>
  <div class="w-48">
    <div class="flex items-center">
      <button
        class="min-w-5"
        style="margin-left: {(path.length - 1) * 1.25}rem;"
        on:click={() => toggleOpen()}
      >
        {#if node.children.length > 0}
          <AngleRightOutline
            class="h-4 transition-transform {open && !ghost ? 'rotate-90' : ''}"
          />
        {/if}
      </button>
      <button
        id="handle-{path.id}"
        class="relative min-w-5"
        draggable={true}
        on:dragstart={(event) => dragStart(event)}
        on:dragend={(event) => dragEnd(event)}
      >
        <BarsOutline class="h-4" />
        {#if canDragDrop}
          <div
            id="peer-dropzone-{path.id}"
            class="absolute -left-6 z-50 h-7 w-12 bg-red-400"
            role="table"
            on:dragover={(event) => dragOver(event, "peer")}
            on:dragleave={(event) => dragLeave(event)}
            on:drop={(event) => drop(event, "peer")}
          />
          <div
            id="child-dropzone-{path.id}"
            class="absolute left-6 z-50 h-7 w-12 bg-blue-400"
            role="table"
            on:dragover={(event) => dragOver(event, "child")}
            on:dragleave={(event) => dragLeave(event)}
            on:drop={(event) => drop(event, "child")}
          />
        {/if}
      </button>
      <div>{path.name}</div>
    </div>
  </div>
  <div class="w-96 px-2">
    {#if editedDescription !== null}
      <Input
        size="sm"
        on:blur={() => stopEditing()}
        bind:value={editedDescription}
        autofocus
      />
    {:else}
      <A on:click={() => startEditing()} on:keydown={() => startEditing()}>
        {node.name}
      </A>
    {/if}
  </div>
</div>

{#if interactions.dragged && path.equals(interactions.maybeChild)}
  <svelte:self
    {graph}
    {interactions}
    ghost={true}
    path={path.concat(interactions.dragged.name)}
  />
{/if}

{#if open && !ghost}
  {#each node.children as child}
    <svelte:self {graph} {interactions} path={path.concat(child)} />
  {/each}
{/if}

{#if interactions.dragged && path.equals(interactions.maybePeer)}
  <svelte:self
    {graph}
    {interactions}
    ghost={true}
    path={path.parent().concat(interactions.dragged.name)}
  />
{/if}
