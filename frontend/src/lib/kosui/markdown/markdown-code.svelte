<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";

  let {
    token,
    ...restProps
  }: MarkdownComponentProps<Tokens.Code> & HTMLAttributes<HTMLPreElement> =
    $props();
</script>

<!-- eslint-disable-next-line svelte/no-at-html-tags -->
<pre {...restProps}><code class={token.lang}>{@html token.text}</code></pre>
