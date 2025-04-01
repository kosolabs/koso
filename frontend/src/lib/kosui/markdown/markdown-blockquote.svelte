<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownProps } from ".";

  let { children }: MarkdownProps<Tokens.Blockquote> = $props();
</script>

<blockquote>
  {@render children()}
</blockquote>
