<script lang="ts">
  import { Markdown, MarkdownHeading } from "$lib/kosui/markdown";

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

<Markdown bind:value options={{ breaks: true, gfm: true }}>
  {#snippet heading(token, children)}
    <MarkdownHeading {token} {children} />
  {/snippet}
</Markdown>
