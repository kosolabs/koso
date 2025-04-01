<script lang="ts">
  import type { MarkedToken, Tokens } from "marked";
  import type { MarkdownProps } from ".";
  import { MarkdownTokens } from ".";

  let { token }: MarkdownProps<Tokens.Table> = $props();
</script>

<table>
  <thead>
    <tr>
      {#each token.header as item}
        <th>
          <MarkdownTokens tokens={item.tokens as MarkedToken[]} />
        </th>
      {/each}
    </tr>
  </thead>
  <tbody>
    {#each token.rows as row}
      <tr>
        {#each row as col}
          <td>
            <MarkdownTokens tokens={col.tokens as MarkedToken[]} />
          </td>
        {/each}
      </tr>
    {/each}
  </tbody>
</table>
