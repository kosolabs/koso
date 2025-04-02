<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLAnchorAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";

  let {
    token,
    children,
    ...restProps
  }: MarkdownComponentProps<Tokens.Link> & HTMLAnchorAttributes = $props();
</script>

<a href={token.href} title={token.title} {...restProps}>
  {@render children()}
</a>
