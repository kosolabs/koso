<script lang="ts">
  import { colors, variants } from "$lib/kosui/base";
  import { Button } from "$lib/kosui/button";
  import { toTitleCase } from "$lib/kosui/utils";
  import { Link2 } from "lucide-svelte";

  let clicked = $state(false);
</script>

{#if clicked}
  <div data-testid="result">Clicked!</div>
{/if}

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each variants as variant (variant)}
    {#each colors as color (color)}
      {@const title = toTitleCase(variant) + " " + toTitleCase(color)}
      <div>
        <div class="flex flex-wrap gap-2">
          <Button {variant} {color} onclick={() => (clicked = true)}>
            {title} Button
          </Button>
          <Button
            {variant}
            {color}
            icon={Link2}
            onclick={() => (clicked = true)}
          >
            {title} Icon
          </Button>
          <Button disabled {variant} {color} onclick={() => (clicked = true)}>
            {title} Disabled
          </Button>
        </div>
      </div>
    {/each}
  {/each}
</div>
