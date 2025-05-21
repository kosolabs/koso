<script lang="ts">
  import {
    Markdown,
    MarkdownHeading,
    MarkdownTable,
    MarkdownTableCell,
  } from "$lib/kosui/markdown";
  import MarkdownBlockquote from "$lib/kosui/markdown/markdown-blockquote.svelte";
  import MarkdownCode from "$lib/kosui/markdown/markdown-code.svelte";
  import MarkdownList from "$lib/kosui/markdown/markdown-list.svelte";
  import { twMerge } from "tailwind-merge";

  let value = `
# Heading 1

## Heading 2

### Heading 3

---

> Blockquote with in it.

Markdown text with **strong** text, *em* text, ~del~ text, and \`inline="code"\`.

A paragraph  
with a break.

| Header 1     | Header 2     | Header 3     |
| ------------ | ------------ | ------------ |
| Row 1, Col 1 | Row 1, Col 2 | Row 1, Col 3 |
| Row 2, Col 1 | Row 2, Col 2 | Row 2, Col 3 |
| Row 3, Col 1 | Row 3, Col 2 | Row 3, Col 3 |

1. ol item 1
1. ol item 2

- ul 1
- ul 2
  - sub 2.a
  - sub 2.b

[link](https://google.com)

\`\`\`json
{
  "key": "value"
}
\`\`\`

![Image](https://lh3.googleusercontent.com/a/ACg8ocIRfl1MJrdKF_V8e46SQijmFzs1JoEaQLogCsOEIYC-T2Hk2xcPKw=s96-c)

<p>Inline <em>HTML</em></p>

\\* Escaped bullet

EOF`;
</script>

<Markdown {value} options={{ breaks: true, gfm: true }}>
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
