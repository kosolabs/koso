<script lang="ts">
  import { ResponsiveText } from "$lib/components/ui/responsive-text";
  import { TaskStatusIcon } from ".";

  type Props = {
    done: number;
    total: number;
  };
  const { done, total }: Props = $props();
</script>

<div
  class="flex items-center gap-2"
  title="{done} of {total} ({Math.round((done * 100) / total)}%)"
>
  <TaskStatusIcon status={done / total} />
  {#if done === total}
    <ResponsiveText>Done</ResponsiveText>
  {:else if done === 0}
    <ResponsiveText>Not Started</ResponsiveText>
  {:else}
    <ResponsiveText>In Progress</ResponsiveText>
  {/if}
</div>
