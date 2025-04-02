<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLDelAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";

  let {
    children,
    ...restProps
  }: MarkdownComponentProps<Tokens.Del> & HTMLDelAttributes = $props();
</script>

<del {...restProps}>
  {@render children()}
</del>
