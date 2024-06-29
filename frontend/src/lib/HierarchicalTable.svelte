<script lang="ts">
  import { AngleRightOutline, BarsOutline } from "flowbite-svelte-icons";
  import type { Task } from "./task";

  export let tasks: Map<string, Task>;
  export let rootTaskId: string;

  type RenderedTask = {
    task: Task;
    path: string[];
    ghost: boolean;
  };

  let draggedTask: RenderedTask | null = null;
  let maybePeerTask: RenderedTask | null = null;
  let maybeChildTask: RenderedTask | null = null;

  let expanded = new Map<string, boolean>();
  function isExpanded(task: RenderedTask): boolean {
    return expanded.get(task.path.join("-")) ?? true;
  }
  function toggleExpanded(task: RenderedTask) {
    expanded = expanded.set(task.path.join("-"), !isExpanded(task));
  }

  function hasCycle(parent: string, child: string): boolean {
    if (child === parent) {
      return true;
    }
    for (const next of tasks.get(child)!.children) {
      if (hasCycle(parent, next)) {
        return true;
      }
    }
    return false;
  }

  function isValid(parent: RenderedTask, child: RenderedTask): boolean {
    if (hasCycle(parent.task.id, child.task.id)) {
      return false;
    }
    if (tasks.get(parent.task.id)?.children.includes(child.task.id)) {
      return false;
    }
    return true;
  }

  function dragstart(event: DragEvent, task: RenderedTask) {
    draggedTask = task;
    event.dataTransfer!.effectAllowed = "linkMove";
    const taskId = task.path.join("-");

    document.getElementById(`peer-dropzone-${taskId}`)!.hidden = true;
    document.getElementById(`child-dropzone-${taskId}`)!.hidden = true;

    const rowEl = document.getElementById(`row-${taskId}`)!;
    const handleEl = document.getElementById(`handle-${taskId}`)!;
    const rowRect = rowEl.getBoundingClientRect();
    const handleRect = handleEl.getBoundingClientRect();

    event.dataTransfer!.setDragImage(
      rowEl,
      handleRect.x - rowRect.x + event.offsetX,
      handleRect.y - rowRect.y + event.offsetY,
    );
  }

  function dragover(
    event: DragEvent,
    draggedOverTask: RenderedTask,
    relationship: "peer" | "child",
  ) {
    event.preventDefault();
    if (draggedTask === null) {
      return;
    }
    if (!isValid(draggedOverTask, draggedTask)) {
      return;
    }
    if (relationship === "child") {
      maybeChildTask = draggedOverTask;
    } else if (relationship === "peer") {
      maybePeerTask = draggedOverTask;
    }
  }

  function dragleave(event: DragEvent) {
    event.preventDefault();
    maybeChildTask = null;
    maybePeerTask = null;
  }

  function drop(
    event: DragEvent,
    droppedTask: RenderedTask,
    relationship: "peer" | "child",
  ) {
    event.preventDefault();
    if (draggedTask === null) {
      return;
    }
    if (!isValid(droppedTask, draggedTask)) {
      return;
    }
    if (relationship === "child") {
      const parent = tasks.get(droppedTask.task.id)!;
      parent.children.splice(0, 0, draggedTask.task.id);
      tasks = tasks;
    } else if (relationship === "peer") {
      const parentId = droppedTask.path[droppedTask.path.length - 2];
      const parent = tasks.get(parentId)!;
      const index = parent.children.indexOf(droppedTask.task.id);
      parent.children.splice(index + 1, 0, draggedTask.task.id);
      tasks = tasks;
    }
  }

  function dragend(event: DragEvent, task: RenderedTask) {
    event.preventDefault();
    const taskId = task.path.join("-");
    draggedTask = null;
    maybeChildTask = null;
    maybePeerTask = null;
    document.getElementById(`peer-dropzone-${taskId}`)!.hidden = false;
    document.getElementById(`child-dropzone-${taskId}`)!.hidden = false;
  }

  let renderedTasks: RenderedTask[] = [];
  function flatten(task: Task, parent: string[]) {
    const path = parent.concat(task.id);
    const renderedTask: RenderedTask = {
      task,
      path,
      ghost: false,
    };
    renderedTasks.push(renderedTask);
    if (
      draggedTask &&
      maybeChildTask &&
      maybeChildTask.path.join("-") == renderedTask.path.join("-")
    ) {
      renderedTasks.push({
        task: draggedTask.task,
        path: path.concat(draggedTask.task.id),
        ghost: true,
      });
    }
    if (isExpanded(renderedTask)) {
      for (const childId of task.children) {
        const child = tasks.get(childId)!;
        flatten(child, path);
      }
    }
    if (
      draggedTask &&
      maybePeerTask &&
      maybePeerTask.path.join("-") === renderedTask.path.join("-")
    ) {
      renderedTasks.push({
        task: draggedTask.task,
        path: parent.concat(draggedTask.task.id),
        ghost: true,
      });
    }
  }
  $: {
    expanded, maybeChildTask, maybePeerTask;
    renderedTasks = [];
    flatten(tasks.get(rootTaskId)!, []);
  }
</script>

<h1 class="my-8 text-4xl">Yotei Hierarchical Table</h1>

<div>
  <div id="header" class="rounded border text-xs font-bold uppercase">
    <div class="my-1 flex items-center rounded p-2">
      <div class="border-r" style="width: 12rem">
        <div class="flex items-center">
          <div class="w-5"></div>
          <div class="w-5"></div>
          <div>ID</div>
        </div>
      </div>
      <div class="w-40 px-2">Description</div>
    </div>
  </div>

  <div
    id="body"
    class="[&>*:nth-child(even)]:bg-gray-100 [&>*:nth-child(odd)]:bg-gray-200"
  >
    {#each renderedTasks as renderedTask}
      {@const taskId = renderedTask.path.join("-")}
      {@const expanded = isExpanded(renderedTask)}
      <div
        id="row-{taskId}"
        class="my-1 flex items-center rounded p-2"
        style="opacity: {renderedTask.ghost ? 0.5 : 1};"
      >
        <div style="width: 12rem">
          <div class="flex items-center">
            <button
              class="w-5"
              style="margin-left: {(renderedTask.path.length - 1) * 1.25}rem;"
              on:click={() => toggleExpanded(renderedTask)}
            >
              {#if renderedTask.task.hasChildren()}
                <AngleRightOutline
                  class="h-4 transition-transform"
                  style="transform:rotate({expanded ? '90' : '0'}deg)"
                />
              {/if}
            </button>
            <button
              id="handle-{taskId}"
              class="relative w-5"
              draggable={true}
              on:dragstart={(event) => dragstart(event, renderedTask)}
              on:dragend={(event) => dragend(event, renderedTask)}
            >
              <BarsOutline class="h-4" />
              <div
                id="peer-dropzone-{taskId}"
                class="absolute -left-6 z-50 h-7 w-12"
                role="table"
                on:dragover={(event) => dragover(event, renderedTask, "peer")}
                on:dragleave={(event) => dragleave(event)}
                on:drop={(event) => drop(event, renderedTask, "peer")}
              />
              <div
                id="child-dropzone-{taskId}"
                class="absolute left-6 z-50 h-7 w-12"
                role="table"
                on:dragover={(event) => dragover(event, renderedTask, "child")}
                on:dragleave={(event) => dragleave(event)}
                on:drop={(event) => drop(event, renderedTask, "child")}
              />
            </button>
            <div>{renderedTask.task.id}</div>
          </div>
        </div>
        <div class="w-40 px-2">{renderedTask.task.title}</div>
      </div>
    {/each}
  </div>
</div>
