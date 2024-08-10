<script lang="ts">
  import { Circle, CircleCheck, CircleFadingArrowUp } from "lucide-svelte";
  import type { Status } from "./koso";

  export let status: Status | null;
</script>

{#if status === "Done"}
  <CircleCheck />
{:else if status == "In Progress"}
  <CircleFadingArrowUp />
{:else}
  <Circle />
{/if}
