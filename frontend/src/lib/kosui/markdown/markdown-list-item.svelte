<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownProps } from ".";

  let { children }: MarkdownProps<Tokens.ListItem> = $props();
</script>

<li>
  {@render children()}
</li>
