<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownComponentProps } from ".";

  let { children }: MarkdownComponentProps<Tokens.Paragraph> = $props();
</script>

<p>
  {@render children()}
</p>
