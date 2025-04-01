<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownProps } from ".";

  let { token }: MarkdownProps<Tokens.Codespan> = $props();
</script>

<code>{token.raw.slice(1, token.raw.length - 1)}</code>
