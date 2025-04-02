<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";

  let {
    children,
    ...restProps
  }: MarkdownComponentProps<Tokens.Strong> & HTMLAttributes<HTMLElement> =
    $props();
</script>

<strong {...restProps}>
  {@render children()}
</strong>
