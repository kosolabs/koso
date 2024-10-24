<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    children: Snippet;
  };
  const { children }: Props = $props();
</script>

<div
  class="flex h-5 min-w-5 items-center justify-center rounded border bg-muted p-1 text-xs text-muted-foreground shadow-sm"
>
  {@render children()}
</div>
