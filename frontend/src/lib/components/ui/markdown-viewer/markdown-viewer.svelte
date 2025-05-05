<script lang="ts">
  import {
    Markdown,
    MarkdownBlockquote,
    MarkdownHeading,
    MarkdownList,
    MarkdownTable,
    MarkdownTableCell,
  } from "$lib/kosui/markdown";
  import type { ClassName } from "$lib/kosui/utils";
  import { twMerge } from "tailwind-merge";
  import MarkdownCode from "./markdown-code.svelte";

  type Props = {
    value: string;
  } & ClassName;
  let { value, class: className }: Props = $props();
</script>

<div class={twMerge("flex flex-col gap-2", className)}>
  <Markdown bind:value options={{ breaks: true, gfm: true }}>
    {#snippet blockquote({ token, children })}
      <MarkdownBlockquote class="border border-l-4 p-2" {token} {children} />
    {/snippet}
    {#snippet code({ token, children })}
      <MarkdownCode class="rounded border p-2 text-sm" {token} {children} />
    {/snippet}
    {#snippet heading({ token, children })}
      <MarkdownHeading
        class={twMerge(
          "text-lg",
          token.depth === 1 && "text-3xl",
          token.depth === 2 && "text-2xl",
          token.depth === 3 && "text-xl",
        )}
        {token}
        {children}
      />
    {/snippet}
    {#snippet list({ token, children })}
      <MarkdownList
        class={twMerge("ml-4", token.ordered ? "list-decimal" : "list-disc")}
        {token}
        {children}
      />
    {/snippet}
    {#snippet table({ token, children })}
      <MarkdownTable class="w-min" {token} {children} />
    {/snippet}
    {#snippet tableCell({ token, children })}
      <MarkdownTableCell
        class="border p-1 whitespace-nowrap"
        {token}
        {children}
      />
    {/snippet}
  </Markdown>
</div>
