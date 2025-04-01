<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownComponentProps } from ".";

  let { token, children }: MarkdownComponentProps<Tokens.Heading> = $props();
</script>

<svelte:element this={`h${token.depth}`}>
  {@render children()}
</svelte:element>
