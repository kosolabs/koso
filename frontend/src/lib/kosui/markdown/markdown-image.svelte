<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownProps } from ".";

  let { token }: MarkdownProps<Tokens.Image> = $props();
</script>

<img src={token.href} title={token.title} alt={token.text} />
