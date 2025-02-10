<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    title: string;
    children: Snippet;
  };
  const { title, children }: Props = $props();
</script>

<div class="flex flex-col gap-2">
  <h1 class="text-2xl font-thin">{title}</h1>
  <hr />
  <div class="py-2">
    {@render children()}
  </div>
</div>
