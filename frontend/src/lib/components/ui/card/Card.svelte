<script lang="ts">
  import { Box } from "kosui";
  import { getProjectContext } from "$lib/dag-table";

  type Props = {
    dupeId: string;
    taskId: string;
    similarity: string;
  };

  let { dupeId, taskId, similarity }: Props = $props();

  const { koso } = getProjectContext();

  const task = koso.getTask(taskId);
</script>

<div class="border-red flex flex-col gap-2 rounded-lg bg-white p-4 shadow-sm">
  <div class="text-sm text-gray-500">Task: {taskId}</div>
  <div>Task Name {task.name}</div>
  <div class="text-lg font-semibold">Similarity: {similarity}</div>
  <!-- <div class="text-gray-600">{taskDescription}</div> -->
  <!-- {#if parentTask}
    <div class="text-xs text-gray-400">Parent: {parentTask}</div>
  {/if} -->
</div>

<div class="flex">
  <div class="flex w-8 justify-center">
    <Box
      variant="filled"
      shape="rounded"
      color="secondary"
      class="cursor-default px-1 text-xs"
    ></Box>
  </div>
</div>
