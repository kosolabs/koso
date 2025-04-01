<script lang="ts">
  import { type Tokens } from "marked";
  import type { MarkdownComponentProps } from ".";

  let { token }: MarkdownComponentProps<Tokens.Escape> = $props();
</script>

{token.text}
