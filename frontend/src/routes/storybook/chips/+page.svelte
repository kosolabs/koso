<script lang="ts">
  import { colors, shapes, variants } from "$lib/kosui/base";
  import { Chip } from "$lib/kosui/chip";
</script>

<div class="flex flex-wrap gap-4 rounded-lg border p-4">
  {#each shapes as shape (shape)}
    {#each variants as variant (variant)}
      {#each colors as color (color)}
        <div>
          <Chip
            {variant}
            {color}
            {shape}
            onClick={() => console.log("click!")}
            onDelete={() => console.log("delete!")}
          >
            Chip
          </Chip>
        </div>
      {/each}
    {/each}
  {/each}
</div>
