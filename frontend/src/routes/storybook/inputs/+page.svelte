<script lang="ts">
  import { baseVariants } from "$lib/kosui/base";
  import { Input, type InputVariants } from "$lib/kosui/input";
  import { toTitleCase } from "$lib/kosui/utils";

  const inputVariantsVariants = Object.keys(
    baseVariants.variants.variant,
  ) as Exclude<InputVariants["variant"], undefined>[];
  const inputVariantsColors = Object.keys(
    baseVariants.variants.color,
  ) as Exclude<InputVariants["color"], undefined>[];
  const inputVariantsSizes = Object.keys(
    baseVariants.variants.scale,
  ) as Exclude<InputVariants["scale"], undefined>[];

  let value: string = $state("");
</script>

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each inputVariantsVariants as variant}
    {#each inputVariantsColors as color}
      {#each inputVariantsSizes as scale}
        {@const title = toTitleCase(variant)}
        <div>
          <div class="mb-2">{title} Inputs ({color}, {scale})</div>
          <div class="flex flex-wrap gap-2">
            <Input
              bind:value
              type="text"
              placeholder="Search"
              name="search"
              autocomplete="off"
              {variant}
              {color}
              {scale}
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
              {scale}
            />
          </div>
        </div>
      {/each}
    {/each}
  {/each}
</div>
