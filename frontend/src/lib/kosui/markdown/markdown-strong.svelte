<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownProps } from ".";

  let { children }: MarkdownProps<Tokens.Strong> = $props();
</script>

<strong>
  {@render children()}
</strong>
