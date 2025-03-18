<script lang="ts">
  import { colors, shapes } from "$lib/kosui/base";
  import { ToggleButton, ToggleGroup } from "$lib/kosui/toggle";
  import { toTitleCase } from "$lib/kosui/utils";

  let value: "1" | "2" | "3" | undefined = $state("2");
</script>

<div>{value}</div>

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each shapes as shape (shape)}
    {#each colors as color (color)}
      {@const title = toTitleCase(shape) + " " + toTitleCase(color)}
      <div>
        <ToggleGroup bind:value>
          <ToggleButton {shape} {color} value="1">
            {title} 1
          </ToggleButton>
          <ToggleButton {shape} {color} value="2">
            {title} 2
          </ToggleButton>
          <ToggleButton {shape} {color} value="3">
            {title} 3
          </ToggleButton>
        </ToggleGroup>
      </div>
    {/each}
  {/each}
</div>
