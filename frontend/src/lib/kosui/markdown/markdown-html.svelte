<script lang="ts">
  import type { Tokens } from "marked";
  import type { MarkdownProps } from ".";

  let { token }: MarkdownProps<Tokens.HTML | Tokens.Tag> = $props();
</script>

<!-- eslint-disable-next-line svelte/no-at-html-tags -->
{@html token.text}
