<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownProps } from ".";

  let { children }: MarkdownProps<Tokens.Paragraph> = $props();
</script>

<p>
  {@render children()}
</p>
