<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLLiAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";

  let {
    children,
    ...restProps
  }: MarkdownComponentProps<Tokens.ListItem> & HTMLLiAttributes = $props();
</script>

<li {...restProps}>
  {@render children()}
</li>
