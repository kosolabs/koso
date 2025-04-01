<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownProps } from ".";

  let { token, children }: MarkdownProps<Tokens.Link> = $props();
</script>

<a href={token.href} title={token.title}>
  {@render children()}
</a>
