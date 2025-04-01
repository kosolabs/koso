<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownProps } from ".";

  let { children }: MarkdownProps<Tokens.Space> = $props();
</script>

{@render children()}
