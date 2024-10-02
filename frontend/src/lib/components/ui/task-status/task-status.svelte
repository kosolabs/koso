<script lang="ts">
  import { ResponsiveText } from "$lib/components/ui/responsive-text";
  import { TaskStatusIcon } from ".";

  type Props = {
    inProgress: number;
    done: number;
    total: number;
  };
  const { inProgress, done, total }: Props = $props();
</script>

<div
  class="flex items-center gap-2"
  title="{done} of {total} ({Math.round((done * 100) / total)}%)"
>
  {#if done === total}
    <TaskStatusIcon status="Done" />
    <ResponsiveText>Done</ResponsiveText>
  {:else if done === 0 && inProgress == 0}
    <TaskStatusIcon status="Not Started" />
    <ResponsiveText>Not Started</ResponsiveText>
  {:else}
    <TaskStatusIcon status="In Progress" />
    <ResponsiveText>In Progress</ResponsiveText>
  {/if}
</div>
