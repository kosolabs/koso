<script lang="ts">
  import { Circle, CircleCheck, CircleFadingArrowUp } from "lucide-svelte";

  export let status: string | null;
</script>

{#if status === "Not Started"}
  <Circle />
{:else if status == "In Progress"}
  <CircleFadingArrowUp />
{:else if status == "Done"}
  <CircleCheck />
{/if}
<div class="text-sm">{status}</div>
