<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = { children: Snippet };
  const { children }: Props = $props();
</script>

<div class="whitespace-nowrap text-sm max-md:hidden">
  {@render children()}
</div>
