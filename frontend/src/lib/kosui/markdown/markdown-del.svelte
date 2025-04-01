<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownComponentProps } from ".";

  let { children }: MarkdownComponentProps<Tokens.Del> = $props();
</script>

<del>
  {@render children()}
</del>
