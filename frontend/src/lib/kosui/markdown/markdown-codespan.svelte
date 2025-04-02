<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";

  let {
    token,
    ...restProps
  }: MarkdownComponentProps<Tokens.Codespan> & HTMLAttributes<HTMLElement> =
    $props();
</script>

<code {...restProps}>
  {token.raw.slice(1, token.raw.length - 1)}
</code>
