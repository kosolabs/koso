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

<pre {...restProps}><code class={`lang-${token.lang}`}>{token.text}</code></pre>
