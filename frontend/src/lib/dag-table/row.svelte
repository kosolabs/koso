<script module lang="ts">
  export type RowType = {
    edit(editing: boolean): void;
    getStatusPosition(): DOMRect;
    linkOrMove(visible: boolean): void;
  };
</script>

<script lang="ts">
  import { auth, type User } from "$lib/auth.svelte";
  import {
    Chip,
    parseChipProps,
    type ChipProps,
  } from "$lib/components/ui/chip";
  import { confetti } from "$lib/components/ui/confetti";
  import { Editable } from "$lib/components/ui/editable";
  import { TaskStatus, TaskStatusSelect } from "$lib/components/ui/task-status";
  import UserSelect from "$lib/components/ui/user-select/user-select.svelte";
  import type { Koso, Node } from "$lib/koso.svelte";
  import { Shortcut } from "$lib/shortcuts";
  import { cn } from "$lib/utils";
  import type { Map } from "immutable";
  import { ChevronRight, Grip } from "lucide-svelte";
  import { getContext } from "svelte";
  import DropIndicator from "./drop-indicator.svelte";
  import LinkCommand from "./link-command.svelte";

  type Props = {
    index: number;
    node: Node;
    users: User[];
  };
  const { index, node, users }: Props = $props();

  const koso = getContext<Koso>("koso");

  let rowElement: HTMLTableRowElement | undefined = $state();
  let idCellElement: HTMLTableCellElement | undefined = $state();
  let handleElement: HTMLButtonElement | undefined = $state();
  let statusElement: HTMLTableCellElement | undefined = $state();

  let dragOverPeer = $state(false);
  let dragOverChild = $state(false);
  let isEditing = $state(false);
  let linkOpen = $state(false);

  let task = $derived(koso.getTask(node.name));
  let reporter = $derived(getUser(users, task.reporter));
  let assignee = $derived(getUser(users, task.assignee));
  let open = $derived(koso.expanded.has(node));
  let isDragging = $derived(node.equals(koso.dragged));
  let isMoving = $derived(isDragging && koso.dropEffect === "move");
  let isHovered = $derived(koso.highlighted === node.name);
  let isSelected = $derived(node.equals(koso.selected));
  let progress = $derived(koso.getProgress(task.id));
  let tags = $derived(getTags(koso.parents));

  $effect(() => {
    if (rowElement && node.equals(koso.selected)) {
      rowElement.focus();
    }
  });

  export function edit(editing: boolean) {
    isEditing = editing;
  }

  export function getStatusPosition(): DOMRect {
    if (!statusElement) throw new Error("Status element is undefined");
    return statusElement.getBoundingClientRect();
  }

  export function linkOrMove(visible: boolean) {
    linkOpen = visible;
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
    koso.highlighted = null;
    koso.selected = null;
    koso.dragged = node;

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
    koso.dragged = null;
  }

  function handleDropNodePeer(event: DragEvent) {
    event.preventDefault();
    if (koso.dragged === null || koso.dropEffect === "none") {
      return;
    }

    const dragDestParent = node.parent.name;
    const dragDestOffset = koso.getOffset(node) + 1;

    if (koso.dropEffect === "copy") {
      koso.linkNode(koso.dragged, dragDestParent, dragDestOffset);
    } else if (koso.dropEffect === "move") {
      koso.moveNode(koso.dragged, dragDestParent, dragDestOffset);
    } else {
      throw new Error(`Invalid dropEffect: ${koso.dropEffect}`);
    }
    koso.dragged = null;
    dragOverPeer = false;
    dragOverChild = false;
  }

  function handleDropNodeChild(event: DragEvent) {
    event.preventDefault();
    if (koso.dragged === null || koso.dropEffect === "none") {
      return;
    }

    const dragDestParent = node.name;
    const dragDestOffset = 0;

    if (koso.dropEffect === "copy") {
      koso.linkNode(koso.dragged, dragDestParent, dragDestOffset);
    } else if (koso.dropEffect === "move") {
      koso.moveNode(koso.dragged, dragDestParent, dragDestOffset);
    } else {
      throw new Error(`Invalid dropEffect: ${koso.dropEffect}`);
    }
    koso.dragged = null;
    dragOverPeer = false;
    dragOverChild = false;
  }

  function handleDragOverPeer(event: DragEvent) {
    event.preventDefault();
    const dataTransfer = event.dataTransfer;
    if (koso.dragged === null || dataTransfer === null) {
      return;
    }

    if (koso.canLink(koso.dragged, node.parent.name)) {
      koso.dropEffect = event.altKey ? "copy" : "move";
      dataTransfer.dropEffect = koso.dropEffect;
      dragOverPeer = true;
    } else if (koso.canMove(koso.dragged, node.parent.name)) {
      dataTransfer.dropEffect = "move";
      koso.dropEffect = "move";
      dragOverPeer = true;
    } else {
      dataTransfer.dropEffect = "none";
      koso.dropEffect = "none";
    }
  }

  function handleDragOverChild(event: DragEvent) {
    event.preventDefault();
    const dataTransfer = event.dataTransfer;
    if (koso.dragged === null || dataTransfer === null) {
      return;
    }

    if (koso.canLink(koso.dragged, node.name)) {
      koso.dropEffect = event.altKey ? "copy" : "move";
      dataTransfer.dropEffect = koso.dropEffect;
      dragOverChild = true;
    } else if (koso.canMove(koso.dragged, node.name)) {
      dataTransfer.dropEffect = "move";
      koso.dropEffect = "move";
      dragOverChild = true;
    } else {
      dataTransfer.dropEffect = "none";
      koso.dropEffect = "none";
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
    if (koso.dragged) return;
    koso.highlighted = node.name;
  }

  function handleUnhighlight() {
    if (koso.dragged) return;
    koso.highlighted = null;
  }

  function handleRowClick(event: MouseEvent) {
    event.preventDefault();
    koso.selected = node;
  }
</script>

<tr
  tabindex="0"
  class={cn(
    "rounded bg-opacity-50 outline outline-2 outline-transparent",
    index % 2 === 0 ? "bg-muted" : "",
    isMoving ? "opacity-50" : "",
    isHovered ? "bg-accent" : "",
    isSelected ? "outline-primary" : "",
  )}
  aria-label={`Task ${task.num}`}
  onmouseout={handleUnhighlight}
  onmouseover={handleHighlight}
  onblur={handleUnhighlight}
  onfocus={handleHighlight}
  onclick={handleRowClick}
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
          onclick={handleToggleOpen}
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
        ondragstart={handleDragStart}
        ondragend={handleDragEnd}
        ondrag={handleDrag}
        bind:this={handleElement}
      >
        <Grip class="w-4" />
        <div class="overflow-x-hidden whitespace-nowrap">{task.num}</div>
      </button>
    </div>
  </td>
  {#if koso.debug}
    <td class={cn("border-l border-t p-2 text-xs lg:text-nowrap")}>
      {task.id}
    </td>
  {/if}
  <td
    class={cn("border-l border-t p-2")}
    onkeydown={(e) => e.stopPropagation()}
    bind:this={statusElement}
  >
    {#if task.children.length === 0}
      <TaskStatusSelect
        value={task.status}
        statusTime={task.statusTime ? new Date(task.statusTime) : null}
        closeFocus={rowElement}
        onselect={(status) => {
          if (status === "Done") confetti.add(getStatusPosition());
          koso.setTaskStatus(node, status, auth.user);
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
        onclick={() => (koso.selected = node)}
        onsave={async (name) => {
          koso.setTaskName(task.id, name);
        }}
        ondone={() => edit(false)}
        onkeydown={(e) => {
          if (
            !Shortcut.INSERT_NODE.matches(e) &&
            !Shortcut.INSERT_CHILD_NODE.matches(e)
          ) {
            e.stopPropagation();
          }
        }}
      />
      <LinkCommand {node} bind:open={linkOpen} closeFocus={rowElement} />
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
      koso.dragged ? "-my-3 h-8 " : "",
      koso.debug ? "bg-primary/20" : "",
    )}
    style="width: {childOffset}px;"
    aria-label={`Task ${task.num} Peer Dropzone`}
    ondragover={handleDragOverPeer}
    ondragenter={handleDragEnterPeer}
    ondragleave={handleDragLeavePeer}
    ondrop={handleDropNodePeer}
  ></button>
  <button
    class={cn(
      "absolute z-50 h-1 cursor-default transition-all",
      koso.dragged ? "-my-3 h-8" : "",
      koso.debug ? "bg-secondary/20" : "",
    )}
    style="width: {cellWidth - childOffset}px; margin-left: {childOffset}px;"
    aria-label={`Task ${task.num} Child Dropzone`}
    ondragover={handleDragOverChild}
    ondragenter={handleDragEnterChild}
    ondragleave={handleDragLeaveChild}
    ondrop={handleDropNodeChild}
  ></button>

  {#if koso.dragged}
    {@const source = koso.getTask(koso.dragged.name)}
    {#if dragOverPeer}
      <DropIndicator
        src={source}
        dest={task}
        width={rowWidth - peerOffset}
        offset={peerOffset}
        type="Peer"
      />
    {/if}
    {#if dragOverChild}
      <DropIndicator
        src={source}
        dest={task}
        width={rowWidth - childOffset}
        offset={childOffset}
        type="Child"
      />
    {/if}
  {/if}
{/if}
