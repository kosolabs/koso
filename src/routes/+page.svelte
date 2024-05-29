<script lang="ts">
  import { Banner } from "flowbite-svelte";
  import Tasker from "$lib/tasker.svelte";

  type Task = {
    id: string;
    name: string;
    children: string[];
  };

  let tasks: Task[] = [
    { id: "task-1", name: "Task 1", children: ["task-2"] },
    { id: "task-2", name: "Task 2", children: [] },
    { id: "task-3", name: "Task 3", children: ["task-2", "task-4"] },
    { id: "task-4", name: "Task 4", children: [] },
  ];
  let tasksById: { [id: string]: Task } = {};
  for (let task of tasks) {
    tasksById[task.id] = task;
  }
</script>

<!-- <Banner id="default-banner" position="absolute">Welcome to Yotei!</Banner> -->

<ul class="border-2 border-red-100">
  {#each tasks as task}
    <Tasker id={task.id} name={task.name} />
    <ul class="mb-1 ml-2 mt-1">
      {#each task.children as childId}
        <Tasker id={tasksById[childId].id} name={tasksById[childId].name} />
      {/each}
    </ul>
  {/each}
</ul>
