<script lang="ts">
  import { CircularProgress } from "$lib/components/ui/circular-progress";
  import type { Status } from "$lib/koso";
  import { CircleCheck, CircleFadingArrowUp } from "lucide-svelte";

  type Props = {
    status: Status | number | null;
  };
  const { status }: Props = $props();
</script>

{#if status === "Done" || status === 1}
  <CircleCheck color="hsl(var(--primary))" />
{:else if status === "In Progress"}
  <CircleFadingArrowUp color="hsl(var(--primary))" />
{:else}
  {@const progress = typeof status === "number" ? status : 0}
  <CircularProgress {progress} color="hsl(var(--primary))">
    {#if progress !== 0}
      {Math.round(progress * 100)}%
    {/if}
  </CircularProgress>
{/if}
