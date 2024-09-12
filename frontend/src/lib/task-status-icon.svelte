<script lang="ts">
  import { Circle, CircleCheck, CircleFadingArrowUp } from "lucide-svelte";
  import CircularProgress from "./circular-progress.svelte";
  import type { Status } from "./koso";

  export let status: Status | number | null;
</script>

{#if status === "Done" || status === 1}
  <CircleCheck color="hsl(var(--primary))" />
{:else if status === "In Progress"}
  <CircleFadingArrowUp color="hsl(var(--primary))" />
{:else if typeof status === "number"}
  <CircularProgress progress={status} fillColor="hsl(var(--primary))">
    {Math.round(status * 100)}%
  </CircularProgress>
{:else}
  <Circle color="hsl(var(--secondary))" />
{/if}
