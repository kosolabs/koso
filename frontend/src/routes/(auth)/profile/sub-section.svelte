<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    title: string;
    children: Snippet;
  };
  const { title, children }: Props = $props();
</script>

<div class="flex flex-col gap-2 rounded border p-2">
  <h2 class="text-xl font-thin">{title}</h2>
  <div>
    {@render children()}
  </div>
</div>
