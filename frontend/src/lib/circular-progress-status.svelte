<script lang="ts">
  import { Circle, CircleCheck } from "lucide-svelte";
  import CircularProgress from "./circular-progress.svelte";
  import ResponsiveText from "./responsive-text.svelte";

  export let done: number;
  export let total: number;
</script>

<div class="flex items-center gap-2">
  {#if done === total}
    <CircleCheck />
    <ResponsiveText title="{done} of {total}">Done</ResponsiveText>
  {:else if done === 0}
    <Circle />
    <ResponsiveText title="{done} of {total}">Not Started</ResponsiveText>
  {:else}
    <CircularProgress progress={done / total}>
      {Math.round((done * 100) / total)}%
    </CircularProgress>
    <ResponsiveText title="{done} of {total}">In Progress</ResponsiveText>
  {/if}
</div>
