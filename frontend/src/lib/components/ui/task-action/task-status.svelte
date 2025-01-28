<script lang="ts">
  import { ResponsiveText } from "$lib/components/ui/responsive-text";
  import { cn } from "$lib/utils";
  import type { YTaskProxy } from "$lib/yproxy";
  import { CircleCheck } from "lucide-svelte";
  import { TaskStatusIcon } from ".";
  import { CircularProgress } from "../circular-progress";
  import type { Koso } from "$lib/dag-table";

  type Props = {
    task: YTaskProxy;
    koso: Koso;
  };
  const { task, koso }: Props = $props();

  let isRollup = $derived(task.children.length > 0 || task.kind == "Rollup");
</script>

{#if isRollup}
  {@const progress = koso.getProgress(task.id)}

  <div
    class="flex items-center gap-2"
    title="{progress.done} of {progress.total} ({Math.round(
      (progress.done * 100) / progress.total,
    )}%)"
    aria-label="task-status"
  >
    {#if progress.done === progress.total}
      <CircleCheck color="hsl(var(--primary))" />
      <ResponsiveText>Done</ResponsiveText>
    {:else if progress.done === 0 && progress.inProgress == 0}
      <CircularProgress progress={0} color="hsl(var(--primary))" />
      <ResponsiveText>Not Started</ResponsiveText>
    {:else}
      <CircularProgress
        progress={progress.done / progress.total}
        color="hsl(var(--primary))"
      >
        {Math.round((progress.done * 100) / progress.total)}%
      </CircularProgress>
      <ResponsiveText>In Progress</ResponsiveText>
    {/if}
  </div>
{:else}
  <div class={cn("flex items-center gap-2")} aria-label="task-status">
    <TaskStatusIcon status={task.status} />
    <ResponsiveText>{task.status || "Not Started"}</ResponsiveText>
  </div>
{/if}
