<script lang="ts">
  import { type Tokens } from "marked";
  import type { MarkdownProps } from ".";

  let { token }: MarkdownProps<Tokens.Escape> = $props();
</script>

{token.text}
