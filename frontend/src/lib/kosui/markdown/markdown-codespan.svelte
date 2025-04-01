<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownComponentProps } from ".";

  let { token }: MarkdownComponentProps<Tokens.Codespan> = $props();
</script>

<code>{token.raw.slice(1, token.raw.length - 1)}</code>
