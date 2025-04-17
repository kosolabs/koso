<script lang="ts">
  import { goto } from "$app/navigation";
  import { type User } from "$lib/auth.svelte";
  import { parseChipProps, type ChipProps } from "$lib/components/ui/chip";
  import { Editable } from "$lib/components/ui/editable";
  import { ManagedTaskIcon } from "$lib/components/ui/managed-task-icon";
  import { toast } from "$lib/components/ui/sonner";
  import { TaskStatus } from "$lib/components/ui/task-status";
  import { UserSelect } from "$lib/components/ui/user-select";
  import { Chip } from "$lib/kosui/chip";
  import { Link } from "$lib/kosui/link";
  import { cn } from "$lib/utils";
  import type { Map } from "immutable";
  import { ChevronRight, Grip } from "lucide-svelte";
  import Awareness, {
    getAwarenessOutline,
    getUniqueUsers,
  } from "./awareness.svelte";
  import DescAction from "./desc-action.svelte";
  import DropIndicator from "./drop-indicator.svelte";
  import { TaskLinkage } from "./koso.svelte";
  import LinkPanel, { type Mode } from "./link-panel.svelte";
  import { getPlanningContext, Node } from "./planning-context.svelte";
  import TaskAction from "./task-action.svelte";

  type Props = {
    index: number;
    node: Node;
    users: User[];
    inboxView: boolean;
  };
  const { index, node, users, inboxView }: Props = $props();

  const planningCtx = getPlanningContext();
  const { koso } = planningCtx;

  let rowElement: HTMLTableRowElement | undefined = $state();
  let idCellElement: HTMLTableCellElement | undefined = $state();
  let handleElement: HTMLButtonElement | undefined = $state();
  let taskStatus = $state<TaskStatus | undefined>();

  let dragOverPeer = $state(false);
  let dragOverChild = $state(false);
  let isEditing = $state(false);
  let linkOpen = $state(false);
  let linkMode: Mode = $state(inboxView ? "block" : "link");

  let task = $derived(koso.getTask(node.name));
  let reporter = $derived(getUser(users, task.reporter));
  let assignee = $derived(getUser(users, task.assignee));
  let open = $derived(planningCtx.expanded.has(node));
  let isDragging = $derived(node.equals(planningCtx.dragged));
  let isMoving = $derived(isDragging && planningCtx.dropEffect === "move");
  let isHovered = $derived(planningCtx.highlighted === node.name);
  let isSelected = $derived(node.equals(planningCtx.selected));
  let awareUsers = $derived(
    getUniqueUsers(koso.awareness.filter((a) => node.equals(a.selected[0]))),
  );
  let tags = $derived(getTags(koso.parents));
  let editable = $derived(koso.isEditable(task.id));

  $effect(() => {
    if (rowElement && isSelected && planningCtx.focus) {
      rowElement.focus();
      planningCtx.focus = false;
    }
  });

  export function edit(editing: boolean) {
    isEditing = editing;
  }

  export function showDoneConfetti() {
    taskStatus?.showDoneConfetti();
  }

  export function linkPanel(visible: boolean, mode: Mode) {
    linkOpen = visible;
    linkMode = mode;
  }

  function getTags(allParents: Map<string, string[]>): ChipProps[] {
    const parents = allParents.get(node.name);
    if (!parents) return [];
    return parents
      .filter((parent) => {
        // Don't output a tag for this node's parent, it's duplicative.
        // Except in the inbox view where nodes are a flat list.
        return parent !== node.parent.name || inboxView;
      })
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

          let targetNode = planningCtx.nodes
            .filter((n) => n.name == node.name && n.parent.name === parent.id)
            // Prefer the least nested linkage of this node under the given parent.
            // i.e. the one closed to the root.
            .minBy((n) => n.path.size);
          if (targetNode) {
            planningCtx.selected = targetNode;
            return;
          }
          const root = planningCtx.nodes.get(0);
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
              planningCtx.isVisible(n.name, planningCtx.showDone)
            ) {
              planningCtx.selected = n;
              // Expand all parents of the selected node to ensure it's visible.
              let t = n.parent;
              while (t.length) {
                planningCtx.expand(t);
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
      planningCtx.expand(node);
    } else {
      planningCtx.collapse(node);
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
    planningCtx.highlighted = null;
    planningCtx.selected = null;
    planningCtx.dragged = node;

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
    planningCtx.dragged = null;
  }

  function handleDropNodePeer(event: DragEvent) {
    event.preventDefault();
    if (planningCtx.dragged === null || planningCtx.dropEffect === "none") {
      return;
    }

    const dragDestParent = node.parent;
    const dragDestOffset = planningCtx.getOffset(node) + 1;

    if (planningCtx.dropEffect === "copy") {
      planningCtx.linkNode(planningCtx.dragged, dragDestParent, dragDestOffset);
    } else if (planningCtx.dropEffect === "move") {
      planningCtx.moveNode(planningCtx.dragged, dragDestParent, dragDestOffset);
    } else {
      throw new Error(`Invalid dropEffect: ${planningCtx.dropEffect}`);
    }
    planningCtx.dragged = null;
    dragOverPeer = false;
    dragOverChild = false;
  }

  function handleDropNodeChild(event: DragEvent) {
    event.preventDefault();
    if (planningCtx.dragged === null || planningCtx.dropEffect === "none") {
      return;
    }

    const dragDestParent = node;
    const dragDestOffset = 0;

    if (planningCtx.dropEffect === "copy") {
      planningCtx.linkNode(planningCtx.dragged, dragDestParent, dragDestOffset);
    } else if (planningCtx.dropEffect === "move") {
      planningCtx.moveNode(planningCtx.dragged, dragDestParent, dragDestOffset);
    } else {
      throw new Error(`Invalid dropEffect: ${planningCtx.dropEffect}`);
    }
    planningCtx.dragged = null;
    dragOverPeer = false;
    dragOverChild = false;
  }

  function handleDragOverPeer(event: DragEvent) {
    event.preventDefault();
    const dataTransfer = event.dataTransfer;
    if (planningCtx.dragged === null || dataTransfer === null) {
      return;
    }

    if (
      koso.canLink(
        new TaskLinkage({
          parentId: node.parent.name,
          id: planningCtx.dragged.name,
        }),
      )
    ) {
      if (planningCtx.canMoveNode(planningCtx.dragged, node.parent)) {
        planningCtx.dropEffect = event.altKey ? "copy" : "move";
      } else {
        planningCtx.dropEffect = "copy";
      }
      dataTransfer.dropEffect = planningCtx.dropEffect;
      dragOverPeer = true;
    } else if (planningCtx.canMoveNode(planningCtx.dragged, node.parent)) {
      dataTransfer.dropEffect = "move";
      planningCtx.dropEffect = "move";
      dragOverPeer = true;
    } else {
      dataTransfer.dropEffect = "none";
      planningCtx.dropEffect = "none";
    }
  }

  function handleDragOverChild(event: DragEvent) {
    event.preventDefault();
    const dataTransfer = event.dataTransfer;
    if (planningCtx.dragged === null || dataTransfer === null) {
      return;
    }

    if (
      koso.canLink(
        new TaskLinkage({ parentId: node.name, id: planningCtx.dragged.name }),
      )
    ) {
      if (planningCtx.canMoveNode(planningCtx.dragged, node)) {
        planningCtx.dropEffect = event.altKey ? "copy" : "move";
      } else {
        planningCtx.dropEffect = "copy";
      }
      dataTransfer.dropEffect = planningCtx.dropEffect;
      dragOverChild = true;
    } else if (planningCtx.canMoveNode(planningCtx.dragged, node)) {
      dataTransfer.dropEffect = "move";
      planningCtx.dropEffect = "move";
      dragOverChild = true;
    } else {
      dataTransfer.dropEffect = "none";
      planningCtx.dropEffect = "none";
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
    if (planningCtx.dragged) return;
    planningCtx.highlighted = node.name;
  }

  function handleUnhighlight() {
    if (planningCtx.dragged) return;
    planningCtx.highlighted = null;
  }

  function handleRowClick(event: MouseEvent) {
    event.preventDefault();
    planningCtx.selected = node;
  }
</script>

<tr
  tabindex="0"
  class={cn(
    "rounded outline-2 outline-transparent",
    index % 2 === 0 && "bg-m3-surface-container/30",
    isMoving && "opacity-50",
    isHovered && "bg-m3-surface-container",
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
            onclick={(event) => {
              event.stopPropagation();
              event.preventDefault();
              goto(`/projects/${koso.projectId}?taskId=${task.id}`);
            }}
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
  <td class={cn("border-t border-l p-2")}>
    <TaskStatus {koso} {task} {inboxView} bind:this={taskStatus} />
  </td>
  <td class={cn("w-full border-t border-l px-2 py-1")}>
    <div class={cn("flex items-center gap-x-1")}>
      {#if koso.isManagedTask(task.id)}
        <ManagedTaskIcon kind={task.yKind ?? ""} />
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
            onclick={() => (planningCtx.selected = node)}
            onsave={async (name) => {
              koso.setTaskName(task.id, name);
            }}
            ondone={() => edit(false)}
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
        <LinkPanel
          {task}
          {koso}
          bind:open={linkOpen}
          bind:mode={linkMode}
          anchorEl={rowElement}
        />
      </div>
    </div>
  </td>
  <td class={cn("border-t px-1")}>
    <div class="flex items-center">
      <DescAction detailPanelRenderer={planningCtx} {task} />
      <TaskAction />
    </div>
  </td>
  <td class={cn("border-t border-l p-2")}>
    <UserSelect
      {users}
      value={assignee}
      {editable}
      onSelect={(user) => {
        koso.setAssignee(task.id, user);
      }}
    />
  </td>
  {#if !inboxView}
    <td class={cn("border-t border-l p-2 max-md:hidden")}>
      <UserSelect
        {users}
        value={reporter}
        {editable}
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
      koso.debug && "bg-m3-primary/20",
    )}
    style:width={`${rowWidth}px`}
    style:height={planningCtx.dragged ? `${rowHeight / 2}px` : "1px"}
    style:margin-top={planningCtx.dragged ? `-${rowHeight / 4}px` : "0"}
    aria-label={`Task ${task.num} Peer Dropzone`}
    ondragover={handleDragOverPeer}
    ondragenter={handleDragEnterPeer}
    ondragleave={handleDragLeavePeer}
    ondrop={handleDropNodePeer}
  ></button>
  <button
    class={cn(
      "absolute z-50 cursor-default transition-all",
      koso.debug && "bg-m3-tertiary/20",
    )}
    style:width={`${rowWidth}px`}
    style:height={planningCtx.dragged ? `${rowHeight / 2}px` : "1px"}
    style:margin-top={planningCtx.dragged
      ? `-${(rowHeight * 3) / 4}px`
      : "-1px"}
    aria-label={`Task ${task.num} Child Dropzone`}
    ondragover={handleDragOverChild}
    ondragenter={handleDragEnterChild}
    ondragleave={handleDragLeaveChild}
    ondrop={handleDropNodeChild}
  ></button>

  {#if planningCtx.dragged}
    {@const source = koso.getTask(planningCtx.dragged.name)}
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
