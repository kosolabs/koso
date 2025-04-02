<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";

  let {
    token,
    children,
    ...restProps
  }: MarkdownComponentProps<Tokens.Heading> &
    HTMLAttributes<HTMLHeadingElement> = $props();
</script>

<svelte:element this={`h${token.depth}`} {...restProps}>
  {@render children()}
</svelte:element>
