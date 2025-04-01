<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownComponentProps } from ".";

  let { token }: MarkdownComponentProps<Tokens.Image> = $props();
</script>

<img src={token.href} title={token.title} alt={token.text} />
