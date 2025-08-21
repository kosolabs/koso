<script lang="ts">
  import { Box } from "kosui";

  type Props = {
    dupeId?: string;
    task1Id?: string;
    task2Id?: string;
    similarity?: string;
  };

  let { dupeId, task1Id, task2Id, similarity }: Props = $props();
</script>

<div class="border-red flex flex-col gap-2 rounded-lg bg-white p-4 shadow-sm">
  <div class="text-sm text-gray-500">Task 1: {task1Id}</div>
  <div class="text-sm text-gray-500">Task 2: {task2Id}</div>
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
