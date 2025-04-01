<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownComponentProps } from ".";

  let { token, children }: MarkdownComponentProps<Tokens.Link> = $props();
</script>

<a href={token.href} title={token.title}>
  {@render children()}
</a>
