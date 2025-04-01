<script lang="ts">
  import { type MarkedToken } from "marked";
  import { getMarkdownContext, MarkdownTokens } from ".";

  export type MarkdownTokenProps = {
    token: MarkedToken;
  };
  let { token }: MarkdownTokenProps = $props();

  const ctx = getMarkdownContext();

  let Renderer = ctx.getRenderer(token.type);
</script>

{#if Renderer}
  <Renderer {token}>
    {#if "tokens" in token && token["tokens"]}
      <MarkdownTokens tokens={token["tokens"] as MarkedToken[]} />
    {:else}
      {token.raw}
    {/if}
  </Renderer>
{/if}
