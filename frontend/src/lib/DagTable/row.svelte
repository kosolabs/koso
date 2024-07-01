<script lang="ts">
  import { AngleRightOutline, BarsOutline } from "flowbite-svelte-icons";
  import type { Graph, Path } from ".";
  import type { Interactions } from "./table.svelte";

  export let graph: Graph;
  export let path: Path;
  export let interactions: Interactions;
  export let ghost = false;

  $: node = graph[path.name]!;

  let open = true;

  function toggleOpen() {
    open = !open;
  }

  function dragStart(event: DragEvent) {
    interactions = { ...interactions, dragged: path };

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
    interactions = { ...interactions, dragged: null };
  }

  function drop(event: DragEvent, relationship: "peer" | "child") {
    event.preventDefault();
    if (interactions.dragged === null) {
      return;
    }
    if (relationship === "child") {
      const parent = graph[path.name]!;
      parent.children.splice(0, 0, interactions.dragged.name);
      graph = graph;
    } else if (relationship === "peer") {
      const parentId = path.parent().name;
      const parent = graph[parentId]!;
      const index = parent.children.indexOf(path.name);
      parent.children.splice(index + 1, 0, interactions.dragged.name);
      graph = graph;
    }
  }

  function dragOver(event: DragEvent, relationship: "peer" | "child") {
    event.preventDefault();
    if (interactions.dragged === null) {
      return;
    }
    if (relationship === "child") {
      interactions = { ...interactions, maybeChild: path };
    } else if (relationship === "peer") {
      interactions = { ...interactions, maybePeer: path };
    }
  }

  function dragLeave(event: DragEvent) {
    event.preventDefault();
    interactions = {
      ...interactions,
      maybeChild: null,
      maybePeer: null,
    };
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

  $: canDragDrop = isValidRelationship(path, interactions.dragged);
</script>

<div
  id="row-{path.id}"
  class="my-1 flex items-center rounded p-2"
  style="opacity: {ghost ? 0.5 : 1};"
>
  <div style="width: 12rem">
    <div class="flex items-center">
      <button
        class="w-5"
        style="margin-left: {(path.length - 1) * 1.25}rem;"
        on:click={() => toggleOpen()}
      >
        {#if node.children.length > 0}
          <AngleRightOutline
            class="h-4 transition-transform"
            style="transform:rotate({open && !ghost ? '90' : '0'}deg)"
          />
        {/if}
      </button>
      <button
        id="handle-{path.id}"
        class="relative w-5"
        draggable={true}
        on:dragstart={(event) => dragStart(event)}
        on:dragend={(event) => dragEnd(event)}
      >
        <BarsOutline class="h-4" />
        {#if canDragDrop}
          <div
            id="peer-dropzone-{path.id}"
            class="absolute -left-6 z-50 h-7 w-12"
            role="table"
            on:dragover={(event) => dragOver(event, "peer")}
            on:dragleave={(event) => dragLeave(event)}
            on:drop={(event) => drop(event, "peer")}
          />
          <div
            id="child-dropzone-{path.id}"
            class="absolute left-6 z-50 h-7 w-12"
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
  <div class="w-40 px-2">{node.title}</div>
</div>

{#if interactions.dragged && path.equals(interactions.maybeChild)}
  <svelte:self
    bind:graph
    bind:interactions
    ghost={true}
    path={path.concat(interactions.dragged.name)}
  />
{/if}

{#if open && !ghost}
  {#each node.children as child}
    <svelte:self bind:graph bind:interactions path={path.concat(child)} />
  {/each}
{/if}

{#if interactions.dragged && path.equals(interactions.maybePeer)}
  <svelte:self
    bind:graph
    bind:interactions
    ghost={true}
    path={path.parent().concat(interactions.dragged.name)}
  />
{/if}
