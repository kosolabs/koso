<script lang="ts">
  import { CircularProgress } from "$lib/components/ui/circular-progress";
  import type { Status } from "$lib/koso.svelte";
  import { CircleCheck, CircleFadingArrowUp } from "lucide-svelte";

  type Props = {
    status: Status | null;
  };
  const { status }: Props = $props();
</script>

{#if status === "Done"}
  <CircleCheck color="hsl(var(--primary))" />
{:else if status === "In Progress"}
  <CircleFadingArrowUp color="hsl(var(--primary))" />
{:else}
  <CircularProgress progress={0} color="hsl(var(--primary))" />
{/if}
