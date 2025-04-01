<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownComponentProps } from ".";

  let { children }: MarkdownComponentProps<Tokens.Strong> = $props();
</script>

<strong>
  {@render children()}
</strong>
