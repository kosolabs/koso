<script lang="ts">
  import type { Koso } from "$lib/koso";
  import { cn } from "$lib/utils";
  import { A, Input, Tooltip } from "flowbite-svelte";
  import { ChevronRight, GripVertical } from "lucide-svelte";
  import { getContext } from "svelte";
  import type { Node } from "../koso";
  import {
    collapsed,
    dragged,
    dropEffect,
    highlighted,
    selected,
  } from "./state";

  export let node: Node;
  export let isGhost: boolean = false;

  let ghostNode: Node | null = null;
  let ghostOffset: number;

  $: task = koso.getTask(node.name);

  const koso = getContext<Koso>("koso");

  $: open = !$collapsed.has(node.id);

  function setOpen(open: boolean) {
    if (open) {
      $collapsed.delete(node.id);
      $collapsed = $collapsed;
    } else {
      $collapsed = $collapsed.add(node.id);
    }
  }

  function toggleOpen() {
    setOpen(!open);
  }

  function isHidden(nodes: Set<string>) {
    for (const collapsed of nodes) {
      if (node.id.startsWith(collapsed + "-")) {
        return true;
      }
    }
    return false;
  }
  $: hidden = isHidden($collapsed);

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
    $highlighted = null;
    $selected = null;
    $dragged = node;
    dataTransfer.setData("text/plain", node.id);
    dataTransfer.effectAllowed = "linkMove";
  }

  function handleDrag(event: DragEvent) {
    event.preventDefault();
  }

  function handleDragEnd(event: DragEvent) {
    event.preventDefault();
    $dragged = null;
  }

  function handleDropNode(event: DragEvent) {
    event.preventDefault();
    if ($dragged === null || ghostNode === null || $dropEffect === "none") {
      return;
    }

    if (!$dragged.isRoot() && $dropEffect === "move") {
      koso.moveNode(
        $dragged.name,
        $dragged.parent().name,
        koso.getOffset($dragged),
        ghostNode.parent().name,
        ghostOffset,
      );
    } else {
      koso.addNode($dragged.name, ghostNode.parent().name, ghostOffset);
    }
    $dragged = null;
    ghostNode = null;
  }

  function handleDragOverPeer(event: DragEvent) {
    event.preventDefault();
    const dataTransfer = event.dataTransfer;
    if ($dragged === null || dataTransfer === null) {
      return;
    }

    if ($dragged.isRoot() || $dragged.parent().equals(node.parent())) {
      dataTransfer.dropEffect = "move";
      $dropEffect = "move";
    } else {
      $dropEffect = dataTransfer.effectAllowed === "link" ? "link" : "move";
    }

    ghostNode = node.parent().concat($dragged.name);
    ghostOffset = koso.getOffset(node) + 1;
  }

  function handleDragEnterPeer(event: DragEvent) {
    event.preventDefault();
    setOpen(false);
  }

  function handleDragOverChild(event: DragEvent) {
    event.preventDefault();
    const dataTransfer = event.dataTransfer;
    if ($dragged === null || dataTransfer === null) {
      return;
    }

    if ($dragged.isRoot() || $dragged.parent().equals(node)) {
      dataTransfer.dropEffect = "move";
      $dropEffect = "move";
    } else {
      $dropEffect = dataTransfer.effectAllowed === "link" ? "link" : "move";
    }

    ghostNode = node.concat($dragged.name);
    ghostOffset = 0;
  }

  function handleDragEnterChild(event: DragEvent) {
    event.preventDefault();
    setOpen(true);
  }

  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    ghostNode = null;
  }

  function handleHighlight() {
    if ($dragged) return;
    $highlighted = node;
  }

  function handleUnhighlight() {
    if ($dragged) return;
    $highlighted = null;
  }

  function handleSelect() {
    $selected = node;
  }

  function hasCycle(parent: string, child: string): boolean {
    if (child === parent) {
      return true;
    }
    for (const next of koso.getChildren(child)) {
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
    return koso.getChildren(parent.name).includes(child.name);
  }

  function isSamePeer(node: Node, dragged: Node): boolean {
    if (dragged.isRoot()) {
      return false;
    }
    if (!node.parent().equals(dragged.parent())) {
      return false;
    }
    return koso.getOffset(node) + 1 === koso.getOffset(dragged);
  }

  function isSameChild(node: Node, dragged: Node): boolean {
    if (dragged.isRoot()) {
      return false;
    }
    if (!node.equals(dragged.parent())) {
      return false;
    }
    return koso.getOffset(dragged) === 0;
  }

  $: dragging = !isGhost && node.equals($dragged);
  $: canDragDropPeer =
    !dragging &&
    !node.isRoot() &&
    $dragged &&
    !isSamePeer(node, $dragged) &&
    !hasChild(node.parent(), $dragged) &&
    !hasCycle(node.parent().name, $dragged.name);
  $: canDragDropChild =
    !dragging &&
    $dragged &&
    !isSameChild(node, $dragged) &&
    !hasChild(node, $dragged) &&
    !hasCycle(node.name, $dragged.name);
  $: isMoving = dragging && $dropEffect === "move";
  $: isSelected = node.equals($selected);
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div
  id="row-{node.id}"
  role="row"
  tabindex="0"
  class={cn(
    "flex items-center border border-transparent p-2",
    isMoving ? "border-red-600 opacity-30" : "",
    isGhost ? "border-green-600 opacity-70" : "",
    $highlighted?.name === node.name ? "border-lime-600" : "",
    isSelected ? "border-primary-600" : "",
    hidden ? "hidden" : "",
  )}
  on:mouseover={handleHighlight}
  on:mouseout={handleUnhighlight}
  on:focus={handleHighlight}
  on:blur={handleUnhighlight}
  on:click={handleSelect}
>
  <div class="min-w-48 overflow-x-clip whitespace-nowrap">
    <div class="flex items-center">
      <button
        class="w-5 transition-transform"
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
        class="relative w-5"
        draggable={true}
        on:dragstart={handleDragStart}
        on:dragend={handleDragEnd}
        on:drag={handleDrag}
      >
        <GripVertical class="h-4" />
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

{#if ghostNode && ((node.equals(ghostNode.parent()) && ghostOffset === 0) || (!node.isRoot() && node
        .parent()
        .equals(ghostNode.parent()) && ghostOffset === koso.getOffset(node) + 1))}
  <svelte:self node={ghostNode} isGhost={true} />
{/if}
