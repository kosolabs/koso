<script lang="ts">
  import { user, type User } from "$lib/auth";
  import {
    Chip,
    parseChipProps,
    type ChipProps,
  } from "$lib/components/ui/chip";
  import { Editable } from "$lib/components/ui/editable";
  import { TaskStatus, TaskStatusSelect } from "$lib/components/ui/task-status";
  import UserSelect from "$lib/components/ui/user-select/user-select.svelte";
  import type { Koso, Node } from "$lib/koso";
  import { Shortcut } from "$lib/shortcuts";
  import { cn } from "$lib/utils";
  import type { Map } from "immutable";
  import { ChevronRight, Grip } from "lucide-svelte";
  import { getContext } from "svelte";

  export let index: number;
  export let node: Node;
  export let users: User[];

  const koso = getContext<Koso>("koso");
  const {
    debug,
    dragged,
    dropEffect,
    editing,
    expanded,
    highlighted,
    selected,
    parents,
  } = koso;

  let rowElement: HTMLTableRowElement | undefined;
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
  $: tags = getTags($parents);

  $: isEditing = isSelected && $editing;

  $: {
    if (rowElement && node.equals($selected)) {
      rowElement.focus();
    }
  }

  function getTags(allParents: Map<string, string[]>): ChipProps[] {
    const parents = allParents.get(node.name);
    if (!parents) return [];
    return parents
      .filter((parent) => parent !== node.parent.name)
      .map((parent) => koso.getTask(parent).name)
      .filter((name) => name.length > 0)
      .map((name) => parseChipProps(name));
  }

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

  function handleDragStart(event: DragEvent) {
    const dataTransfer = event.dataTransfer;
    if (!dataTransfer || !rowElement || !handleElement || !idCellElement) {
      return;
    }
    $highlighted = null;
    $selected = null;
    $dragged = node;

    dataTransfer.setData("text/plain", node.id);
    dataTransfer.effectAllowed = "copyMove";

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

    if ($dropEffect === "copy") {
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

    if ($dropEffect === "copy") {
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
      $dropEffect = event.altKey ? "copy" : "move";
      dataTransfer.dropEffect = $dropEffect;
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
      $dropEffect = event.altKey ? "copy" : "move";
      dataTransfer.dropEffect = $dropEffect;
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
</script>

<tr
  tabindex="0"
  class={cn(
    "rounded bg-opacity-50 outline outline-2 outline-transparent",
    index % 2 === 0 ? "bg-row-even" : "bg-row-odd",
    isMoving ? "opacity-50" : "",
    isHovered ? "bg-accent" : "",
    isSelected ? "outline-primary" : "",
  )}
  aria-label={`Task ${task.num}`}
  on:mouseout={handleUnhighlight}
  on:mouseover={handleHighlight}
  on:blur={handleUnhighlight}
  on:focus={handleHighlight}
  on:click={handleRowClick}
  bind:this={rowElement}
>
  <td class={cn("border-t px-2")} bind:this={idCellElement}>
    <div class="flex items-center">
      <div style="width: {(node.length - 1) * 20}px"></div>
      {#if task.children.length > 0}
        <button
          class={cn("w-4 transition-transform", open ? "rotate-90" : "")}
          title={open ? "Collapse" : "Expand"}
          aria-label={`Task ${task.num} Toggle Expand`}
          on:click={handleToggleOpen}
        >
          <ChevronRight class="w-4" />
        </button>
      {:else}
        <div class="w-4"></div>
      {/if}
      <button
        class="flex items-center gap-1 py-1"
        draggable={true}
        aria-label={`Task ${task.num} Drag Handle`}
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
  <td
    class={cn("border-l border-t p-2")}
    on:keydown={(e) => e.stopPropagation()}
  >
    {#if task.children.length === 0}
      <TaskStatusSelect
        value={task.status}
        statusTime={task.statusTime ? new Date(task.statusTime) : null}
        closeFocus={rowElement}
        onselect={(status) => {
          if (!$user) throw new Error("Unauthenticated");
          koso.setTaskStatus(node, status, $user);
        }}
      />
    {:else}
      <TaskStatus
        inProgress={progress.inProgress}
        done={progress.done}
        total={progress.total}
      />
    {/if}
  </td>
  <td class={cn("w-full border-l border-t px-2")}>
    <div class="flex items-center gap-1">
      {#each tags as tag}
        <Chip {...tag} />
      {/each}
      <Editable
        value={task.name}
        aria-label={`Task ${task.num} Edit Name`}
        editing={isEditing}
        onsave={(name) => koso.setTaskName(task.id, name)}
        ondone={() => ($editing = false)}
        onkeydown={(e) => {
          if (
            !Shortcut.INSERT_NODE.matches(e) &&
            !Shortcut.INSERT_CHILD_NODE.matches(e)
          ) {
            e.stopPropagation();
          }
        }}
      />
    </div>
  </td>
  <td class={cn("border-l border-t p-2")}>
    <UserSelect
      {users}
      closeFocus={rowElement}
      value={assignee}
      onselect={(user) => {
        koso.setAssignee(task.id, user);
      }}
    />
  </td>
  <td class={cn("border-l border-t p-2 max-sm:hidden")}>
    <UserSelect
      {users}
      closeFocus={rowElement}
      value={reporter}
      onselect={(user) => {
        koso.setReporter(task.id, user);
      }}
    />
  </td>
</tr>

{#if rowElement && idCellElement}
  {@const cellWidth = idCellElement.clientWidth}
  {@const rowWidth = rowElement.clientWidth}
  {@const peerOffset = node.length * 20}
  {@const childOffset = (node.length + 1) * 20}

  <button
    class={cn(
      "absolute z-50 h-1 cursor-default transition-all",
      $dragged ? "-my-3 h-8 " : "",
      $debug ? "bg-pink-400 bg-opacity-20" : "",
    )}
    style="width: {childOffset}px;"
    aria-label={`Task ${task.num} Peer Dropzone`}
    on:dragover={handleDragOverPeer}
    on:dragenter={handleDragEnterPeer}
    on:dragleave={handleDragLeavePeer}
    on:drop={handleDropNodePeer}
  ></button>
  <button
    class={cn(
      "absolute z-50 h-1 cursor-default transition-all",
      $dragged ? "-my-3 h-8" : "",
      $debug ? "bg-cyan-400 bg-opacity-20" : "",
    )}
    style="width: {cellWidth - childOffset}px; margin-left: {childOffset}px;"
    aria-label={`Task ${task.num} Child Dropzone`}
    on:dragover={handleDragOverChild}
    on:dragenter={handleDragEnterChild}
    on:dragleave={handleDragLeaveChild}
    on:drop={handleDropNodeChild}
  ></button>

  {#if dragOverPeer}
    <button
      class="absolute -my-[0.125rem] h-1 bg-teal-500"
      style="width: {rowWidth - peerOffset}px; margin-left: {peerOffset}px;"
      aria-label={`Task ${task.num} Peer Drop Indicator`}
    ></button>
  {/if}
  {#if dragOverChild}
    <button
      class="absolute -my-[0.125rem] h-1 bg-teal-500"
      style="width: {rowWidth - childOffset}px; margin-left: {childOffset}px;"
      aria-label={`Task ${task.num} Child Drop Indicator`}
    ></button>
  {/if}
{/if}
