<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownComponentProps } from ".";

  let { children }: MarkdownComponentProps<Tokens.Def> = $props();
</script>

<dfn>
  {@render children()}
</dfn>
