<script lang="ts">
  import {
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from "flowbite-svelte";
  import { AngleRightOutline } from "flowbite-svelte-icons";

  type Task = {
    id: string;
    name: string;
    level: number;
    expanded: boolean;
    visible: boolean;
    hasChildren: boolean;
  };

  let tasks: Task[] = [
    {
      id: "001",
      name: "Task 1",
      level: 0,
      expanded: true,
      visible: true,
      hasChildren: true,
    },
    {
      id: "002",
      name: "Task 1.1",
      level: 1,
      expanded: false,
      visible: true,
      hasChildren: true,
    },
    {
      id: "003",
      name: "Task 1.1.1",
      level: 2,
      expanded: false,
      visible: false,
      hasChildren: false,
    },
    {
      id: "004",
      name: "Task 2",
      level: 0,
      expanded: true,
      visible: true,
      hasChildren: false,
    },
    {
      id: "005",
      name: "Task 3",
      level: 0,
      expanded: true,
      visible: true,
      hasChildren: false,
    },
  ];
  let tasksById: { [id: string]: Task } = {};
  for (let task of tasks) {
    tasksById[task.id] = task;
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

<div id="body" class="[&>*:nth-child(even)]:bg-gray-100">
  {#each tasks as task}
    {#if task.visible}
      <div class="my-2 flex items-center rounded">
        <div class="w-40">
          <div class="flex items-center">
            <div class="w-5" style="margin-left: {task.level * 1.25}rem;">
              {#if task.hasChildren}
                {#if task.expanded}
                  <AngleRightOutline class="h-4 rotate-90" />
                {:else}
                  <AngleRightOutline class="h-4" />
                {/if}
              {/if}
            </div>
            <div>{task.id}</div>
          </div>
        </div>
        <div class="w-40">{task.name}</div>
      </div>
    {/if}
  {/each}
</div>

<h1 class="my-8 text-4xl">Flowbite Table</h1>

<Table hoverable={true}>
  <TableHead>
    <TableHeadCell>ID</TableHeadCell>
    <TableHeadCell>Description</TableHeadCell>
  </TableHead>
  <TableBody tableBodyClass="divide-y">
    {#each tasks as task}
      {#if task.visible}
        <TableBodyRow>
          <TableBodyCell>{task.id}</TableBodyCell>
          <TableBodyCell>{task.name}</TableBodyCell>
        </TableBodyRow>
      {/if}
    {/each}
  </TableBody>
</Table>
