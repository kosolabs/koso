<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownProps } from ".";

  let { children }: MarkdownProps<Tokens.Def> = $props();
</script>

<dfn>
  {@render children()}
</dfn>
