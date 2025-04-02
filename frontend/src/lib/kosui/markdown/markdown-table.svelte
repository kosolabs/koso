<script lang="ts">
  import type { MarkedToken, Tokens } from "marked";
  import type { HTMLTableAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";
  import { MarkdownTokens } from ".";

  let {
    token,
    ...restProps
  }: MarkdownComponentProps<Tokens.Table> & HTMLTableAttributes = $props();
</script>

<table {...restProps}>
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
