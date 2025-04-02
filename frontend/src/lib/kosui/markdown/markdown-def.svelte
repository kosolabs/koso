<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";

  let {
    children,
    ...restProps
  }: MarkdownComponentProps<Tokens.Def> & HTMLAttributes<HTMLElement> =
    $props();
</script>

<dfn {...restProps}>
  {@render children()}
</dfn>
