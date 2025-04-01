<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownProps } from ".";

  let { children }: MarkdownProps<Tokens.Em> = $props();
</script>

<em>
  {@render children()}
</em>
