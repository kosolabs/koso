<script lang="ts">
  import type { User } from "$lib/auth";
  import type { Koso } from "$lib/koso";
  import UserSelect from "$lib/user-select.svelte";
  import TaskStatusSelect from "$lib/task-status-select.svelte";
  import TaskStatus from "$lib/task-status.svelte";
  import { cn } from "$lib/utils";
  import { A, Avatar, Dropdown, Input, Tooltip } from "flowbite-svelte";
  import {
    ChevronRight,
    GripVertical,
    Circle,
    CircleCheck,
    CircleFadingArrowUp,
  } from "lucide-svelte";
  import { getContext } from "svelte";
  import type { Node } from "../koso";
  import {
    collapsed,
    dragged,
    dropEffect,
    hidden,
    highlighted,
    selected,
  } from "./state";

  export let index: number;
  export let node: Node;
  export let isGhost: boolean = false;
  export let users: User[];
  export let rowCallback: (el: HTMLDivElement) => void = () => {};

  let assigneeSelectorOpen: boolean = false;
  let reporterSelectorOpen: boolean = false;
  let statusSelectorOpen: boolean = false;
  let element: HTMLDivElement | undefined;
  let ghostNode: Node | null = null;
  let ghostOffset: number;

  function row(el: HTMLDivElement) {
    element = el;
    rowCallback(el);
  }

  function getUser(users: User[], email: string | null): User | null {
    for (const user of users) {
      if (user.email === email) {
        return user;
      }
    }
    return null;
  }

  $: task = koso.getTask(node.name);
  $: reporter = getUser(users, task.reporter);
  $: assignee = getUser(users, task.assignee);

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

  function handleToggleOpen(event: MouseEvent) {
    event.stopPropagation();
    setOpen(!open);
  }

  let editedTaskName: string | null = null;

  function handleStartEditingTaskName(event: MouseEvent | CustomEvent) {
    event.stopPropagation();
    $selected = node;
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

  function handleFocus(event: FocusEvent) {
    event.preventDefault();
    $selected = node;
  }

  function handleRowClick(event: MouseEvent) {
    event.preventDefault();
    $selected = node;
  }

  function handleRowKeydown(event: KeyboardEvent) {
    if (!element) throw new Error("Reference to focusable div is undefined");

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
      element.blur();
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (event.key === "ArrowLeft") {
      setOpen(false);
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (event.key === "ArrowRight") {
      setOpen(true);
      event.preventDefault();
      event.stopPropagation();
      return;
    }
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

  $: isDragging = !isGhost && node.equals($dragged);
  $: canDragDropPeer =
    !isDragging &&
    !node.isRoot() &&
    $dragged &&
    !isSamePeer(node, $dragged) &&
    !hasChild(node.parent(), $dragged) &&
    !hasCycle(node.parent().name, $dragged.name);
  $: canDragDropChild =
    !isDragging &&
    $dragged &&
    !isSameChild(node, $dragged) &&
    !hasChild(node, $dragged) &&
    !hasCycle(node.name, $dragged.name);
  $: isMoving = isDragging && $dropEffect === "move";
  $: isHovered = $highlighted?.name === node.name;
  $: isSelected = node.equals($selected);
  $: isHidden = $hidden.has(node.id);
</script>

<tr
  id="row-{node.id}"
  tabindex="0"
  class={cn(
    "rounded border outline-none",
    index % 2 === 0 ? "bg-slate-50" : "bg-white",
    isMoving ? "bg-red-200 opacity-50" : "",
    isGhost ? "bg-green-200 opacity-70" : "",
    isHovered ? "bg-primary-50" : "",
    isSelected ? "bg-primary-200 outline-primary-400" : "",
    isSelected ? "outline outline-2" : "",
    isHidden ? "hidden" : "",
  )}
  on:mouseout={handleUnhighlight}
  on:mouseover={handleHighlight}
  on:blur={() => {}}
  on:focus={handleFocus}
  on:click={handleRowClick}
  on:keydown={handleRowKeydown}
  use:row
>
  <td class={cn("border p-2", isSelected ? "border-transparent" : "")}>
    <div class="flex items-center">
      <div style="width: {(node.length - 1) * 1.25}rem;" />
      {#if task.children.length > 0}
        <button
          class="w-4 transition-transform"
          class:rotate-90={open && !isGhost}
          on:click={handleToggleOpen}
        >
          <ChevronRight class="w-4" />
        </button>
        <Tooltip class="text-nowrap">
          {open ? "Collapse" : "Expand"}
        </Tooltip>
      {:else}
        <div class="w-4" />
      {/if}
      <button
        class="relative w-4"
        draggable={true}
        on:dragstart={handleDragStart}
        on:dragend={handleDragEnd}
        on:drag={handleDrag}
      >
        <GripVertical class="w-4" />
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
      <div class="overflow-x-hidden whitespace-nowrap">{task.num}</div>
    </div>
  </td>
  <td class={cn("border p-2", isSelected ? "border-transparent" : "")}>
    {#if editedTaskName !== null}
      <Input
        size="sm"
        class="p-1"
        on:click={(event) => event.stopPropagation()}
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
  </td>
  <td class={cn("border p-2", isSelected ? "border-transparent" : "")}>
    <button class="flex gap-1">
      <Avatar src={assignee?.picture || ""} rounded size="xs" />
      <div class="max-md:hidden">{assignee?.name || "Unassigned"}</div>
    </button>
    <Dropdown bind:open={assigneeSelectorOpen}>
      <UserSelect
        {users}
        on:select={(event) => {
          koso.setAssignee(task.id, event.detail);
          assigneeSelectorOpen = false;
        }}
      />
    </Dropdown>
  </td>
  <td
    class={cn(
      "border p-2 max-md:hidden",
      isSelected ? "border-transparent" : "",
    )}
  >
    <button class="flex gap-1">
      <Avatar src={reporter?.picture || ""} rounded size="xs" />
      <div>{reporter?.name || "Unknown"}</div>
    </button>
    <Dropdown bind:open={reporterSelectorOpen}>
      <UserSelect
        {users}
        on:select={(event) => {
          koso.setReporter(task.id, event.detail);
          reporterSelectorOpen = false;
        }}
      />
    </Dropdown>
  </td>
  <td class={cn("border p-2", isSelected ? "border-transparent" : "")}>
    <button class="flex gap-1">
      <TaskStatus status={task.status || "Not Started"} />
    </button>
    <Dropdown bind:open={statusSelectorOpen}>
      <TaskStatusSelect
        on:select={(event) => {
          koso.editTaskStatus(task.id, event.detail);
          statusSelectorOpen = false;
        }}
      />
    </Dropdown>
  </td>
</tr>

{#if ghostNode}
  <svelte:self index={index + 1} node={ghostNode} {users} isGhost={true} />
{/if}
