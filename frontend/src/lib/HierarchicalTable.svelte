<script lang="ts">
  import { AngleRightOutline } from "flowbite-svelte-icons";
  import type { Task } from "./task";

  export let tasks: Map<string, Task>;
  export let rootTaskId: string;

  type FlattenedTask = {
    id: string;
    title: string;
    level: number;
  };

  let expanded = new Map<string, boolean>();
  for (const [taskId, _] of tasks) {
    expanded.set(taskId, false);
  }

  function toggleExpanded(taskId: string) {
    expanded = expanded.set(taskId, !expanded.get(taskId));
  }

  let flattenedTasks: FlattenedTask[] = [];
  function flatten(task: Task, depth: number) {
    flattenedTasks.push({
      id: task.id,
      title: task.title,
      level: depth,
    });
    if (expanded.get(task.id)!) {
      for (const childId of task.children) {
        const child = tasks.get(childId)!;
        flatten(child, depth + 1);
      }
    }
  }
  $: {
    expanded;
    flattenedTasks = [];
    flatten(tasks.get(rootTaskId)!, 0);
  }
</script>

<h1 class="my-8 text-4xl">Yotei Hierarchical Table</h1>

<div id="header" class="text-xs font-bold uppercase">
  <div class="my-2 flex items-center rounded">
    <div class="w-40">
      <div class="flex items-center">
        <div class="w-5"></div>
        <div>ID</div>
      </div>
    </div>
    <div class="w-40">Description</div>
  </div>
</div>

<div
  id="body"
  class="[&>*:nth-child(even)]:bg-gray-100 [&>*:nth-child(odd)]:bg-gray-200"
>
  {#each flattenedTasks as task}
    {@const hasChildren = tasks.get(task.id)?.hasChildren()}
    {@const isExpanded = expanded.get(task.id)}
    <div class="my-1 flex items-center rounded">
      <div class="w-40">
        <div class="flex items-center">
          <button
            class="w-5"
            style="margin-left: {task.level * 1.25}rem;"
            on:click={() => toggleExpanded(task.id)}
          >
            {#if hasChildren}
              <AngleRightOutline
                class="h-4 transition-transform"
                style="transform:rotate({isExpanded ? '90' : '0'}deg)"
              />
            {/if}
          </button>
          <div>{task.id}</div>
        </div>
      </div>
      <div class="w-40">{task.title}</div>
    </div>
  {/each}
</div>
