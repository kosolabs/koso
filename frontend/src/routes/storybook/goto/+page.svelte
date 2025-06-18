<script lang="ts">
  import { colors, shapes, underlines, variants } from "$lib/kosui/base";
  import { Goto } from "$lib/kosui/goto";
  import { toTitleCase } from "$lib/kosui/utils";
  import { Link } from "@lucide/svelte";

  let clicked = $state(false);
</script>

{#if clicked}
  <div data-testid="result">Clicked!</div>
{/if}

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each underlines as underline (underline)}
    {@const underlineTitle = toTitleCase(underline)}
    <div>
      <div class="mb-2">Underline {underlineTitle}</div>
      <ul>
        {#each colors as color (color)}
          <li>
            Here is a <Goto variant="text" {underline} {color} href="">
              {color}
            </Goto>
            goto.
          </li>
        {/each}
      </ul>
    </div>
  {/each}

  {#each shapes as shape (shape)}
    {#each variants as variant (variant)}
      {#each colors as color (color)}
        {#each underlines as underline (underline)}
          {@const title =
            toTitleCase(variant) +
            " " +
            toTitleCase(color) +
            " " +
            toTitleCase(shape) +
            " " +
            toTitleCase(underline)}
          <div>
            <div class="flex flex-wrap items-start gap-2">
              <Goto
                href="/storybook/goto"
                {variant}
                {color}
                {shape}
                {underline}
                class="px-2"
              >
                {title}
              </Goto>
              <Goto
                href="/storybook/goto"
                {variant}
                {color}
                {shape}
                {underline}
                class="aspect-square h-9"
              >
                <Link size={16} />
              </Goto>
            </div>
          </div>
        {/each}
      {/each}
    {/each}
  {/each}
</div>
