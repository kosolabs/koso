<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";

  let {
    children,
    token: _token,
    ...restProps
  }: MarkdownComponentProps<Tokens.Paragraph> &
    HTMLAttributes<HTMLParagraphElement> = $props();
</script>

<p {...restProps}>
  {@render children()}
</p>
