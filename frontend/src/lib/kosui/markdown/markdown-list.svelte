<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownProps } from ".";
  import MarkdownTokens from "./markdown-tokens.svelte";

  let { token }: MarkdownProps<Tokens.List> = $props();
</script>

<svelte:element this={token.ordered ? "ol" : "ul"} start={token.start || 1}>
  <MarkdownTokens tokens={token.items} />
</svelte:element>
