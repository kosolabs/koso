<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLOlAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";
  import MarkdownTokens from "./markdown-tokens.svelte";

  let {
    token,
    ...restProps
  }: MarkdownComponentProps<Tokens.List> & HTMLOlAttributes = $props();
</script>

<svelte:element
  this={token.ordered ? "ol" : "ul"}
  start={token.start || 1}
  {...restProps}
>
  <MarkdownTokens tokens={token.items} />
</svelte:element>
