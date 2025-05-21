<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLTableAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";
  import { getMarkdownContext } from ".";

  let {
    token,
    ...restProps
  }: MarkdownComponentProps<Tokens.Table> & HTMLTableAttributes = $props();

  const ctx = getMarkdownContext();
</script>

{#snippet children()}{/snippet}

<table {...restProps}>
  <thead>
    <tr>
      {#each token.header as item (item)}
        {@render ctx.tableCellRenderer({ token: item, children })}
      {/each}
    </tr>
  </thead>
  <tbody>
    {#each token.rows as row (row)}
      <tr>
        {#each row as cell (cell)}
          {@render ctx.tableCellRenderer({ token: cell, children })}
        {/each}
      </tr>
    {/each}
  </tbody>
</table>
