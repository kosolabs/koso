<script lang="ts">
  import { goto } from "$app/navigation";
  import { type User } from "$lib/auth.svelte";
  import { parseChipProps, type ChipProps } from "$lib/components/ui/chip";
  import { Editable } from "$lib/components/ui/editable";
  import { ManagedTaskIcon } from "$lib/components/ui/managed-task-icon";
  import { toast } from "$lib/components/ui/sonner";
  import TaskAction from "$lib/components/ui/task-action/task-action.svelte";
  import { UserSelect } from "$lib/components/ui/user-select";
  import { Chip } from "$lib/kosui/chip";
  import { Link } from "$lib/kosui/link";
  import { INSERT_CHILD_NODE, INSERT_NODE } from "$lib/shortcuts";
  import { cn } from "$lib/utils";
  import type { Map } from "immutable";
  import { ChevronRight, Grip } from "lucide-svelte";
  import { getContext } from "svelte";
  import { Node, type Koso } from ".";
  import Awareness, {
    getAwarenessOutline,
    getUniqueUsers,
  } from "./awareness.svelte";
  import DropIndicator from "./drop-indicator.svelte";
  import LinkPanel from "./link-panel.svelte";

  type Props = {
    index: number;
    node: Node;
    users: User[];
    inboxView: boolean;
  };
  const { index, node, users, inboxView }: Props = $props();

  const koso = getContext<Koso>("koso");

  let rowElement: HTMLTableRowElement | undefined = $state();
  let idCellElement: HTMLTableCellElement | undefined = $state();
  let handleElement: HTMLButtonElement | undefined = $state();
  let taskAction = $state<TaskAction | undefined>();

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
  let awareUsers = $derived(
    getUniqueUsers(koso.awareness.filter((a) => node.equals(a.selected[0]))),
  );
  let tags = $derived(getTags(koso.parents));
  let editable = $derived(koso.isEditable(task.id));

  $effect(() => {
    if (rowElement && isSelected && koso.focus) {
      rowElement.focus();
      koso.focus = false;
    }
  });

  export function edit(editing: boolean) {
    isEditing = editing;
  }

  export function showDoneConfetti() {
    taskAction?.showDoneConfetti();
  }

  export function linkPanel(visible: boolean) {
    linkOpen = visible;
  }

  function getTags(allParents: Map<string, string[]>): ChipProps[] {
    const parents = allParents.get(node.name);
    if (!parents) return [];
    return parents
      .filter((parent) => parent !== node.parent.name)
      .map((parent) => koso.getTask(parent))
      .filter((parent) => parent.name.length > 0)
      .map((parent) => {
        const props: ChipProps = parseChipProps(parent.name);
        if (!inboxView && koso.canUnlink(node.name, parent.id)) {
          props.onDelete = (event) => {
            console.log(event);
            event.stopPropagation();
            koso.unlink(node.name, parent.id);
          };
        }

        props.onClick = (event) => {
          event.stopPropagation();

          if (inboxView) {
            goto(`/projects/${koso.projectId}?taskId=${parent.id}`);
            return;
          }

          let targetNode = koso.nodes
            .filter((n) => n.name == node.name && n.parent.name === parent.id)
            // Prefer the least nested linkage of this node under the given parent.
            // i.e. the one closed to the root.
            .minBy((n) => n.path.size);
          if (targetNode) {
            koso.selected = targetNode;
            return;
          }
          const root = koso.nodes.get(0);
          if (!root) throw new Error("Missing root");

          // All instances of parent are under collapsed nodes or aren't visible.
          // Do a BFS to find the least nested instance.
          let queue: Node[] = [root];
          while (queue.length > 0) {
            let n = queue.shift();
            if (!n) throw new Error("Unexpectly found nothing in queue.");
            if (
              n.name === node.name &&
              n.parent.name === parent.id &&
              koso.isVisible(n, koso.showDone)
            ) {
              koso.selected = n;
              // Expand all parents of the selected node to ensure it's visible.
              let t = n.parent;
              while (t.length) {
                koso.expand(t);
                t = t.parent;
              }
              return;
            }
            for (const child of koso.getChildren(n.name)) {
              queue.push(n.child(child));
            }
          }

          console.log(
            `No parent found. ${parent.id} must not be visible or not in this view.`,
          );
          toast.info(`Could not navigate to "${props.title}"`);
        };
        return props;
      });
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

    const dragDestParent = node.parent;
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

    const dragDestParent = node;
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

    if (koso.canLink(koso.dragged.name, node.parent.name)) {
      if (koso.canMoveNode(koso.dragged, node.parent)) {
        koso.dropEffect = event.altKey ? "copy" : "move";
      } else {
        koso.dropEffect = "copy";
      }
      dataTransfer.dropEffect = koso.dropEffect;
      dragOverPeer = true;
    } else if (koso.canMoveNode(koso.dragged, node.parent)) {
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

    if (koso.canLink(koso.dragged.name, node.name)) {
      if (koso.canMoveNode(koso.dragged, node)) {
        koso.dropEffect = event.altKey ? "copy" : "move";
      } else {
        koso.dropEffect = "copy";
      }
      dataTransfer.dropEffect = koso.dropEffect;
      dragOverChild = true;
    } else if (koso.canMoveNode(koso.dragged, node)) {
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
    "bg-m3-surface-container rounded-m3 outline-2 outline-transparent",
    index % 2 === 0 && "bg-m3-surface-container-low",
    isMoving && "opacity-50",
    isHovered && "bg-m3-surface-container-high",
    getAwarenessOutline(awareUsers),
    dragOverChild && "outline-m3-primary",
    isSelected && "outline-m3-primary",
  )}
  aria-label={`Task ${task.num}`}
  data-testid={`Row ${node.id}`}
  onmouseout={handleUnhighlight}
  onmouseover={handleHighlight}
  onblur={handleUnhighlight}
  onfocus={handleHighlight}
  onclick={handleRowClick}
  bind:this={rowElement}
>
  <td class={cn("border-t px-2")} bind:this={idCellElement}>
    <div class="flex items-center">
      {#if !inboxView}
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
      {:else}
        <div class="overflow-x-hidden whitespace-nowrap">
          <Link
            href={`/projects/${koso.projectId}?taskId=${task.id}`}
            onclick={(event) => event.stopPropagation()}
          >
            {task.num}
          </Link>
        </div>
      {/if}
    </div>
  </td>
  {#if koso.debug}
    <td class={cn("border-t border-l p-2 text-xs lg:text-nowrap")}>
      {task.id}
    </td>
  {/if}
  <td
    class={cn("border-t border-l p-2")}
    onkeydown={(e) => e.stopPropagation()}
  >
    <TaskAction {node} {koso} bind:this={taskAction} />
  </td>
  <td class={cn("w-full border-t border-l px-2 py-1")}>
    <div class={cn("flex items-center gap-x-1")}>
      {#if koso.isManagedTask(task.id)}
        <ManagedTaskIcon kind={task.kind ?? ""} />
      {/if}
      <div class="flex w-full flex-wrap-reverse gap-x-1">
        {#if tags.length > 0}
          <div class="flex flex-wrap items-center gap-x-1">
            {#each tags as { title, description, onClick, onDelete }}
              <Chip color="tertiary" title={description} {onClick} {onDelete}>
                {title}
              </Chip>
            {/each}
          </div>
        {/if}

        {#if editable}
          <Editable
            value={task.name}
            aria-label={`Task ${task.num} Edit Name`}
            editing={isEditing}
            closeFocus={rowElement}
            onclick={() => (koso.selected = node)}
            onsave={async (name) => {
              koso.setTaskName(task.id, name);
            }}
            ondone={() => edit(false)}
            onkeydown={(e) => {
              if (!INSERT_NODE.matches(e) && !INSERT_CHILD_NODE.matches(e)) {
                e.stopPropagation();
              }
            }}
          />
        {:else}
          <Link
            class={cn(
              "h-auto p-0 text-left text-sm text-wrap whitespace-normal",
              task.url ? "text" : "",
            )}
            aria-label={`Task ${task.num} Name`}
            onclick={() => {
              if (!task.url) throw new Error(`No URL set on task ${task}`);
              window.open(task.url, "_blank")!.focus();
            }}
            disabled={!task.url}
            underline="none"
          >
            {task.name || "Untitled"}
          </Link>
        {/if}
        <LinkPanel {node} bind:open={linkOpen} closeFocus={rowElement} />
      </div>
    </div>
  </td>
  <td
    class={cn("border-t border-l p-2")}
    onkeydown={(e) => e.stopPropagation()}
  >
    <UserSelect
      {users}
      value={assignee}
      {editable}
      onOpenChange={() => (koso.selected = node)}
      onSelect={(user) => {
        koso.setAssignee(task.id, user);
      }}
    />
  </td>
  {#if !inboxView}
    <td
      class={cn("border-t border-l p-2 max-md:hidden")}
      onkeydown={(e) => e.stopPropagation()}
    >
      <UserSelect
        {users}
        value={reporter}
        {editable}
        onOpenChange={() => (koso.selected = node)}
        onSelect={(user) => {
          koso.setReporter(task.id, user);
        }}
      />
    </td>
  {/if}
  <td class={cn("relative m-0 w-0 p-0")}>
    <Awareness users={awareUsers} />
  </td>
</tr>

{#if rowElement && idCellElement}
  {@const rowWidth = rowElement.clientWidth}
  {@const rowHeight = rowElement.clientHeight}
  {@const peerOffset = node.length * 20}
  {@const childOffset = (node.length + 1) * 20}

  <button
    class={cn(
      "absolute z-50 cursor-default transition-all",
      koso.debug && "bg-primary/20",
    )}
    style:width={`${rowWidth}px`}
    style:height={koso.dragged ? `${rowHeight / 2}px` : "1px"}
    style:margin-top={koso.dragged ? `-${rowHeight / 4}px` : "0"}
    aria-label={`Task ${task.num} Peer Dropzone`}
    ondragover={handleDragOverPeer}
    ondragenter={handleDragEnterPeer}
    ondragleave={handleDragLeavePeer}
    ondrop={handleDropNodePeer}
  ></button>
  <button
    class={cn(
      "absolute z-50 cursor-default transition-all",
      koso.debug && "bg-secondary/20",
    )}
    style:width={`${rowWidth}px`}
    style:height={koso.dragged ? `${rowHeight / 2}px` : "1px"}
    style:margin-top={koso.dragged ? `-${(rowHeight * 3) / 4}px` : "-1px"}
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
        height={rowHeight}
        width={rowWidth - peerOffset}
        offset={peerOffset}
        type="Peer"
      />
    {/if}
    {#if dragOverChild}
      <DropIndicator
        src={source}
        dest={task}
        height={rowHeight}
        width={rowWidth - childOffset}
        offset={childOffset}
        type="Child"
      />
    {/if}
  {/if}
{/if}
