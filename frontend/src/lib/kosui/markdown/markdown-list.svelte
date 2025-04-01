<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownComponentProps } from ".";
  import MarkdownTokens from "./markdown-tokens.svelte";

  let { token }: MarkdownComponentProps<Tokens.List> = $props();
</script>

<svelte:element this={token.ordered ? "ol" : "ul"} start={token.start || 1}>
  <MarkdownTokens tokens={token.items} />
</svelte:element>
