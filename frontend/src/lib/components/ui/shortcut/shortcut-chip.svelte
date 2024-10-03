<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    children: Snippet;
  };
  const { children }: Props = $props();
</script>

<div
  class="flex h-5 min-w-5 items-center justify-center rounded border bg-secondary p-1 text-xs shadow-sm"
>
  {@render children()}
</div>
