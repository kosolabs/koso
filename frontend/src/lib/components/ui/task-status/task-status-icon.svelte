<script lang="ts">
  import { CircularProgress } from "$lib/kosui/progress";
  import type { Status } from "$lib/yproxy";
  import {
    CircleCheck,
    CircleDotDashed,
    CircleFadingArrowUp,
    OctagonPause,
  } from "lucide-svelte";

  type Props = {
    status: Status | null;
  };
  const { status }: Props = $props();
</script>

{#if status === "Done"}
  <CircleCheck class="text-m3-primary" />
{:else if status === "In Progress"}
  <CircleFadingArrowUp class="text-m3-primary" />
{:else if status === "Ready"}
  <CircleDotDashed class="text-m3-primary" />
{:else if status === "Blocked"}
  <OctagonPause class="text-m3-primary" />
{:else}
  <CircularProgress progress={0} class="text-m3-primary" />
{/if}
