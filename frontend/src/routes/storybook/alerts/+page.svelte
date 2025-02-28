<script lang="ts">
  import { Alert } from "$lib/kosui/alert";
  import { colors, shapes, variants } from "$lib/kosui/base";
</script>

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each shapes as shape (shape)}
    {#each variants as variant (variant)}
      {#each colors as color (color)}
        <div>
          <Alert {variant} {color} {shape}>
            This is an alert with (variant, color, shape): ({variant}, {color}, {shape})
          </Alert>
        </div>
      {/each}
    {/each}
  {/each}
</div>
