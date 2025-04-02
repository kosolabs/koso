<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLImgAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";

  let {
    token,
    ...restProps
  }: MarkdownComponentProps<Tokens.Image> & HTMLImgAttributes = $props();
</script>

<img src={token.href} title={token.title} alt={token.text} {...restProps} />
