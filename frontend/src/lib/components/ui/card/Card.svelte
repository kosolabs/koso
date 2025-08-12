<script>
  export let title: String;
  export let description: String;
  export let taskID: String;
  export let parentTask: String;
</script>

<div class="flex flex-col gap-2 rounded-lg border bg-white p-4 shadow-sm">
  <div class="text-sm text-gray-500">Task ID: {taskID}</div>
  <div class="text-lg font-semibold">{title}</div>
  <div class="text-gray-600">{description}</div>
  {#if parentTask}
    <div class="text-xs text-gray-400">Parent: {parentTask}</div>
  {/if}
</div>
