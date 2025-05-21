<script lang="ts">
  import type { MarkedToken } from "marked";
  import { MarkdownToken } from ".";

  export type MarkdownTokensProps = {
    tokens: MarkedToken[];
  };
  let { tokens }: MarkdownTokensProps = $props();
</script>

{#each tokens as token (token)}
  <MarkdownToken {token} />
{/each}
