<script lang="ts">
  import type { User } from "$lib/auth";
  import CircularProgressStatus from "$lib/circular-progress-status.svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import type { Koso } from "$lib/koso";
  import TaskStatusSelect from "$lib/task-status-select.svelte";
  import UserSelect from "$lib/user-select.svelte";
  import { cn } from "$lib/utils";
  import { ChevronRight, Grip } from "lucide-svelte";
  import { getContext } from "svelte";
  import type { Node } from "../koso";

  export let index: number;
  export let node: Node;
  export let users: User[];
  export let row: (el: HTMLDivElement) => void;

  const koso = getContext<Koso>("koso");
  const { debug, dragged, dropEffect, expanded, highlighted, selected } = koso;

  let rowElement: HTMLDivElement | undefined;
  let idCellElement: HTMLTableCellElement | undefined;
  let handleElement: HTMLButtonElement | undefined;

  let dragOverPeer = false;
  let dragOverChild = false;

  $: task = koso.getTask(node.name);
  $: reporter = getUser(users, task.reporter);
  $: assignee = getUser(users, task.assignee);
  $: open = $expanded.has(node);
  $: isDragging = node.equals($dragged);
  $: isMoving = isDragging && $dropEffect === "move";
  $: isHovered = $highlighted === node.name;
  $: isSelected = node.equals($selected);
  $: progress = koso.getProgress(task.id);

  function getUser(users: User[], email: string | null): User | null {
    for (const user of users) {
      if (user.email === email) {
        return user;
      }
    }
    return null;
  }

  function setOpen(open: boolean) {
    if (open) {
      koso.expand(node);
    } else {
      koso.collapse(node);
    }
  }

  function handleToggleOpen(event: MouseEvent) {
    event.stopPropagation();
    setOpen(!open);
  }

  let editedTaskName: string | null = null;

  function handleStartEditingTaskName(event: MouseEvent | KeyboardEvent) {
    event.stopPropagation();
    event.preventDefault();
    $selected = node;
    editedTaskName = task.name;
  }

  function saveEditedTaskName() {
    if (editedTaskName === null) {
      return;
    }
    koso.setTaskName(node.name, editedTaskName);
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
      revertEditedTaskName();
      $selected = node;
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (event.key === "Enter") {
      saveEditedTaskName();
      $selected = node;
      event.preventDefault();
      event.stopPropagation();
      return;
    }
  }

  function handleDragStart(event: DragEvent) {
    const dataTransfer = event.dataTransfer;
    if (!dataTransfer || !rowElement || !handleElement || !idCellElement) {
      return;
    }
    $highlighted = null;
    $selected = null;
    $dragged = node;

    dataTransfer.setData("text/plain", node.id);
    dataTransfer.effectAllowed = "linkMove";

    const rowRect = rowElement.getBoundingClientRect();
    const handleRect = handleElement.getBoundingClientRect();

    dataTransfer.setDragImage(
      idCellElement,
      handleRect.x - rowRect.x + event.offsetX,
      handleRect.y - rowRect.y + event.offsetY,
    );
  }

  function handleDrag(event: DragEvent) {
    event.preventDefault();
  }

  function handleDragEnd(event: DragEvent) {
    event.preventDefault();
    $dragged = null;
  }

  function handleDropNodePeer(event: DragEvent) {
    event.preventDefault();
    if ($dragged === null || $dropEffect === "none") {
      return;
    }

    const dragDestParent = node.parent.name;
    const dragDestOffset = koso.getOffset(node) + 1;

    if ($dropEffect === "link") {
      koso.linkNode($dragged, dragDestParent, dragDestOffset);
    } else if ($dropEffect === "move") {
      koso.moveNode($dragged, dragDestParent, dragDestOffset);
    } else {
      throw new Error(`Invalid dropEffect: ${$dropEffect}`);
    }
    $dragged = null;
    dragOverPeer = false;
    dragOverChild = false;
  }

  function handleDropNodeChild(event: DragEvent) {
    event.preventDefault();
    if ($dragged === null || $dropEffect === "none") {
      return;
    }

    const dragDestParent = node.name;
    const dragDestOffset = 0;

    if ($dropEffect === "link") {
      koso.linkNode($dragged, dragDestParent, dragDestOffset);
    } else if ($dropEffect === "move") {
      koso.moveNode($dragged, dragDestParent, dragDestOffset);
    } else {
      throw new Error(`Invalid dropEffect: ${$dropEffect}`);
    }
    $dragged = null;
    dragOverPeer = false;
    dragOverChild = false;
  }

  function handleDragOverPeer(event: DragEvent) {
    event.preventDefault();
    const dataTransfer = event.dataTransfer;
    if ($dragged === null || dataTransfer === null) {
      return;
    }

    if (koso.canLink($dragged, node.parent.name)) {
      $dropEffect = dataTransfer.effectAllowed === "link" ? "link" : "move";
      dragOverPeer = true;
    } else if (koso.canMove($dragged, node.parent.name)) {
      dataTransfer.dropEffect = "move";
      $dropEffect = "move";
      dragOverPeer = true;
    } else {
      dataTransfer.dropEffect = "none";
      $dropEffect = "none";
    }
  }

  function handleDragOverChild(event: DragEvent) {
    event.preventDefault();
    const dataTransfer = event.dataTransfer;
    if ($dragged === null || dataTransfer === null) {
      return;
    }

    if (koso.canLink($dragged, node.name)) {
      $dropEffect = dataTransfer.effectAllowed === "link" ? "link" : "move";
      dragOverChild = true;
    } else if (koso.canMove($dragged, node.name)) {
      dataTransfer.dropEffect = "move";
      $dropEffect = "move";
      dragOverChild = true;
    } else {
      dataTransfer.dropEffect = "none";
      $dropEffect = "none";
    }
  }

  let closeTimeout: number | undefined;

  function handleDragEnterPeer(event: DragEvent) {
    event.preventDefault();
    closeTimeout = window.setTimeout(() => setOpen(false), 500);
  }

  function handleDragLeavePeer(event: DragEvent) {
    event.preventDefault();
    clearTimeout(closeTimeout);
    dragOverPeer = false;
  }

  let openTimeout: number | undefined;

  function handleDragEnterChild(event: DragEvent) {
    event.preventDefault();
    openTimeout = window.setTimeout(() => setOpen(true), 500);
  }

  function handleDragLeaveChild(event: DragEvent) {
    event.preventDefault();
    clearTimeout(openTimeout);
    dragOverChild = false;
  }

  function handleHighlight() {
    if ($dragged) return;
    $highlighted = node.name;
  }

  function handleUnhighlight() {
    if ($dragged) return;
    $highlighted = null;
  }

  function handleRowClick(event: MouseEvent) {
    event.preventDefault();
    $selected = node;
  }

  function handleRowKeydown(event: KeyboardEvent) {
    if (!rowElement) throw new Error("Reference to focusable div is undefined");

    if (editedTaskName !== null) {
      return;
    }

    if (event.key === "Enter") {
      editedTaskName = task.name;
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (event.key === "Escape") {
      $selected = null;
      rowElement.blur();
      event.preventDefault();
      event.stopPropagation();
      return;
    }
  }
</script>

<tr
  id="row/{node.id}"
  tabindex="0"
  class={cn(
    "rounded bg-opacity-50",
    index % 2 === 0 ? "bg-row-even" : "bg-row-odd",
    isMoving ? "opacity-50" : "",
    isHovered ? "bg-accent" : "",
    isSelected ? "outline outline-2 outline-primary" : "",
  )}
  on:mouseout={handleUnhighlight}
  on:mouseover={handleHighlight}
  on:blur={handleUnhighlight}
  on:focus={handleHighlight}
  on:click={handleRowClick}
  on:keydown={handleRowKeydown}
  bind:this={rowElement}
  use:row
>
  <td class={cn("border-t px-2")} bind:this={idCellElement}>
    <div class="flex items-center">
      <div style="width: {(node.length - 1) * 20}px" />
      {#if task.children.length > 0}
        <button
          class="w-4 transition-transform"
          class:rotate-90={open}
          title={open ? "Collapse" : "Expand"}
          on:click={handleToggleOpen}
        >
          <ChevronRight class="w-4" />
        </button>
      {:else}
        <div class="w-4" />
      {/if}
      <button
        class="flex items-center gap-1 py-1"
        draggable={true}
        on:dragstart={handleDragStart}
        on:dragend={handleDragEnd}
        on:drag={handleDrag}
        bind:this={handleElement}
      >
        <Grip class="w-4" />
        <div class="overflow-x-hidden whitespace-nowrap">{task.num}</div>
      </button>
    </div>
  </td>
  {#if $debug}
    <td class={cn("border-l border-t p-2 text-xs")}>
      {task.id}
    </td>
  {/if}
  <td class={cn("border-l border-t p-2")}>
    {#if task.children.length === 0}
      <TaskStatusSelect
        value={task.status}
        on:select={(event) => {
          koso.setTaskStatus(task.id, event.detail);
        }}
      />
    {:else}
      <CircularProgressStatus done={progress.numer} total={progress.denom} />
    {/if}
  </td>
  <td class={cn("border-l border-t p-2")}>
    {#if editedTaskName !== null}
      <Input
        class="h-auto bg-transparent p-1"
        on:click={(event) => event.stopPropagation()}
        on:blur={handleEditedTaskNameBlur}
        on:keydown={handleEditedTaskNameKeydown}
        bind:value={editedTaskName}
        autofocus
      />
    {:else}
      <Button
        variant="link"
        class="h-auto text-wrap p-0 text-left hover:no-underline"
        on:click={handleStartEditingTaskName}
        on:keydown={handleStartEditingTaskName}
      >
        {task.name || "Click to edit"}
      </Button>
    {/if}
  </td>
  <td class={cn("border-l border-t p-2")}>
    <UserSelect
      {users}
      value={assignee}
      on:select={(event) => {
        koso.setAssignee(task.id, event.detail);
      }}
    />
  </td>
  <td class={cn("border-l border-t p-2 max-sm:hidden")}>
    <UserSelect
      {users}
      value={reporter}
      on:select={(event) => {
        koso.setReporter(task.id, event.detail);
      }}
    />
  </td>
</tr>

{#if rowElement && idCellElement}
  {@const cellWidth = idCellElement.clientWidth}
  {@const rowWidth = rowElement.clientWidth}
  {@const peerOffset = node.length * 20}
  {@const childOffset = (node.length + 1) * 20}

  {#if $dragged}
    <div
      class={cn(
        "absolute z-50 -my-3 h-8",
        $debug ? "bg-pink-400 bg-opacity-20" : "",
      )}
      style="width: {childOffset}px;"
      role="table"
      on:dragover={handleDragOverPeer}
      on:dragenter={handleDragEnterPeer}
      on:dragleave={handleDragLeavePeer}
      on:drop={handleDropNodePeer}
    />
    <div
      class={cn(
        "absolute z-50 -my-3 h-8",
        $debug ? "bg-cyan-400 bg-opacity-20" : "",
      )}
      style="width: {cellWidth - childOffset}px; margin-left: {childOffset}px;"
      role="table"
      on:dragover={handleDragOverChild}
      on:dragenter={handleDragEnterChild}
      on:dragleave={handleDragLeaveChild}
      on:drop={handleDropNodeChild}
    />
  {/if}

  {#if dragOverPeer}
    <div
      class="absolute -my-[0.125rem] h-1 bg-teal-500"
      style="width: {rowWidth - peerOffset}px; margin-left: {peerOffset}px;"
    />
  {/if}
  {#if dragOverChild}
    <div
      class="absolute -my-[0.125rem] h-1 bg-teal-500"
      style="width: {rowWidth - childOffset}px; margin-left: {childOffset}px;"
    />
  {/if}
{/if}
