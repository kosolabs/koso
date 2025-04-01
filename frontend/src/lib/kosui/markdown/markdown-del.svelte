<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownProps } from ".";

  let { children }: MarkdownProps<Tokens.Del> = $props();
</script>

<del>
  {@render children()}
</del>
