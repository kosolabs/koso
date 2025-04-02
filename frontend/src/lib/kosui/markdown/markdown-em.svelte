<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";

  let {
    children,
    ...restProps
  }: MarkdownComponentProps<Tokens.Em> & HTMLAttributes<HTMLElement> = $props();
</script>

<em {...restProps}>
  {@render children()}
</em>
