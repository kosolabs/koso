<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = { children: Snippet };
  const { children }: Props = $props();
</script>

<div class="text-sm whitespace-nowrap max-md:hidden">
  {@render children()}
</div>
