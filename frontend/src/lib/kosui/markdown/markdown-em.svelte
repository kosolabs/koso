<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownComponentProps } from ".";

  let { children }: MarkdownComponentProps<Tokens.Em> = $props();
</script>

<em>
  {@render children()}
</em>
