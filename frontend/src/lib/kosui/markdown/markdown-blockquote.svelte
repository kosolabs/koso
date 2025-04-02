<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLQuoteAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";

  let {
    children,
    ...restProps
  }: MarkdownComponentProps<Tokens.Blockquote> & HTMLQuoteAttributes = $props();
</script>

<blockquote {...restProps}>
  {@render children()}
</blockquote>
