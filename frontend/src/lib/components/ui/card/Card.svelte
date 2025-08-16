<script lang="ts">
  import { Box } from "kosui";

  type Props = {
    taskID: value;
    taskName: string;
    taskDescription: string;
    parentTask: string;
  };

  let { taskID, taskName, taskDescription, parentTask }: Props = $props();
</script>

<div class="flex flex-col gap-2 rounded-lg border bg-white p-4 shadow-sm">
  <div class="text-sm text-gray-500">Task ID: {taskID}</div>
  <div class="text-lg font-semibold">{taskName}</div>
  <div class="text-gray-600">{taskDescription}</div>
  {#if parentTask}
    <div class="text-xs text-gray-400">Parent: {parentTask}</div>
  {/if}
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
