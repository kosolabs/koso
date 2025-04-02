<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownComponentProps } from ".";

  let { token }: MarkdownComponentProps<Tokens.HTML | Tokens.Tag> = $props();
</script>

<!-- eslint-disable-next-line svelte/no-at-html-tags -->
{@html token.text}
