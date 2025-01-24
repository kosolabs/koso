<script lang="ts">
  import { CircularProgress } from "$lib/components/ui/circular-progress";
  import { ResponsiveText } from "$lib/components/ui/responsive-text";
  import { CircleCheck } from "lucide-svelte";

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
  aria-label="task-status"
>
  {#if done === total}
    <CircleCheck color="hsl(var(--primary))" />
    <ResponsiveText>Done</ResponsiveText>
  {:else if done === 0 && inProgress == 0}
    <CircularProgress progress={0} color="hsl(var(--primary))" />
    <ResponsiveText>Not Started</ResponsiveText>
  {:else}
    <CircularProgress progress={done / total} color="hsl(var(--primary))">
      {Math.round((done * 100) / total)}%
    </CircularProgress>
    <ResponsiveText>In Progress</ResponsiveText>
  {/if}
</div>
