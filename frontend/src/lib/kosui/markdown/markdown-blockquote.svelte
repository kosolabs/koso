<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownComponentProps } from ".";

  let { children }: MarkdownComponentProps<Tokens.Blockquote> = $props();
</script>

<blockquote>
  {@render children()}
</blockquote>
