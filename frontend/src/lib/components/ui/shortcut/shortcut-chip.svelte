<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    children: Snippet;
  };
  const { children }: Props = $props();
</script>

<div
  class="bg-muted text-muted-foreground flex h-5 min-w-5 items-center justify-center rounded border p-1 text-xs shadow-xs"
>
  {@render children()}
</div>
