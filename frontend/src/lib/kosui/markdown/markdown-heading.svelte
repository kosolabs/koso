<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownProps } from ".";

  let { token, children }: MarkdownProps<Tokens.Heading> = $props();
</script>

<svelte:element this={`h${token.depth}`}>
  {@render children()}
</svelte:element>
