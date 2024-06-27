<script lang="ts">
  import HierarchicalTable from "$lib/HierarchicalTable.svelte";
  import { Task } from "$lib/task";

  let tasks = new Map<string, Task>([
    ["000", new Task("000", "Root", ["001", "004", "005"])],
    ["001", new Task("001", "Task 1", ["002"])],
    ["002", new Task("002", "Task 1.1", ["003"])],
    ["003", new Task("003", "Task 1.1.1", [])],
    ["004", new Task("004", "Task 2", [])],
    ["005", new Task("005", "Task 3", [])],
  ]);
</script>

<HierarchicalTable bind:tasks rootTaskId="000" />
