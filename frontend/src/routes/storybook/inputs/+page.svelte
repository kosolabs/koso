<script lang="ts">
  import { colors, variants } from "$lib/kosui/base";
  import { Input } from "$lib/kosui/input";
  import { toTitleCase } from "$lib/kosui/utils";

  let value: string = $state("");
</script>

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each variants as variant (variant)}
    {#each colors as color (color)}
      {@const title = toTitleCase(variant)}
      <div>
        <div class="mb-2">{title} Inputs ({color})</div>
        <div class="flex flex-wrap gap-2">
          <Input
            bind:value
            type="text"
            placeholder="Search"
            name="search"
            autocomplete="off"
            {variant}
            {color}
          />
          <Input
            disabled
            bind:value
            type="text"
            placeholder="Search"
            name="search"
            autocomplete="off"
            {variant}
            {color}
          />
        </div>
      </div>
    {/each}
  {/each}
</div>
