<script lang="ts">
  import hljs from "highlight.js";
  import type { Tokens } from "marked";
  import type { HTMLAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";
  import "./theme.css";

  let {
    token,
    ...restProps
  }: MarkdownComponentProps<Tokens.Code> & HTMLAttributes<HTMLPreElement> =
    $props();

  const code = $derived(
    hljs.highlight(token.text, { language: token.lang || "plaintext" }).value,
  );
</script>

<!-- eslint-disable-next-line svelte/no-at-html-tags -->
<pre {...restProps}><code class={`lang-${token.lang}`}>{@html code}</code></pre>
