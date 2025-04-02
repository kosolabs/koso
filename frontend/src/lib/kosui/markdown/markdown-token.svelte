<script lang="ts">
  import { type MarkedToken } from "marked";
  import { getMarkdownContext, MarkdownTokens } from ".";

  export type MarkdownTokenProps = {
    token: MarkedToken;
  };
  let { token }: MarkdownTokenProps = $props();

  const ctx = getMarkdownContext();

  let renderer = ctx.getRenderer(token.type);
</script>

{#snippet children()}
  {#if "tokens" in token && token["tokens"]}
    <MarkdownTokens tokens={token["tokens"] as MarkedToken[]} />
  {:else}
    {token.raw}
  {/if}
{/snippet}

{#if renderer}
  {@render renderer({ token, children })}
{/if}
