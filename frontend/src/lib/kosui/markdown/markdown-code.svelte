<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownProps } from ".";

  let { token }: MarkdownProps<Tokens.Code> = $props();
</script>

<pre><code class={`lang-${token.lang}`}>{token.text}</code></pre>
