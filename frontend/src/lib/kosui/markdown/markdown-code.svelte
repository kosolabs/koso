<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownComponentProps } from ".";

  let { token }: MarkdownComponentProps<Tokens.Code> = $props();
</script>

<pre><code class={`lang-${token.lang}`}>{token.text}</code></pre>
