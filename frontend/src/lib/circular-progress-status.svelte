<script lang="ts">
  import ResponsiveText from "./responsive-text.svelte";
  import TaskStatusIcon from "./task-status-icon.svelte";

  export let done: number;
  export let total: number;
</script>

<div class="flex items-center gap-2">
  <TaskStatusIcon status={done / total} />
  {#if done === total}
    <ResponsiveText title="{done} of {total}">Done</ResponsiveText>
  {:else if done === 0}
    <ResponsiveText title="{done} of {total}">Not Started</ResponsiveText>
  {:else}
    <ResponsiveText title="{done} of {total}">In Progress</ResponsiveText>
  {/if}
</div>
