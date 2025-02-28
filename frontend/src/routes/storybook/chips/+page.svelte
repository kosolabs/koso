<script lang="ts">
  import { colors, shapes, variants } from "$lib/kosui/base";
  import { Chip } from "$lib/kosui/chip";
</script>

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each shapes as shape}
    {#each variants as variant}
      {#each colors as color}
        <div>
          <Chip
            {variant}
            {color}
            {shape}
            onDelete={() => console.log("delete!")}>Chip</Chip
          >
        </div>
      {/each}
    {/each}
  {/each}
</div>
