<script lang="ts">
  import { colors, shapes } from "$lib/kosui/base";
  import { Toggle, ToggleButton, ToggleGroup } from "$lib/kosui/toggle";
  import { toTitleCase } from "$lib/kosui/utils";

  let value: "1" | "2" | "3" | undefined = $state("2");
</script>

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each colors as color (color)}
    {@const title = toTitleCase(color)}
    <div>
      <div class="flex flex-wrap gap-2">
        <Toggle {color} pressed={value === "1"} onclick={() => (value = "1")}>
          {title} Toggle 1
        </Toggle>
        <Toggle {color} pressed={value === "2"} onclick={() => (value = "2")}>
          {title} Toggle 2
        </Toggle>
        <Toggle {color} pressed={value === "3"} onclick={() => (value = "3")}>
          {title} Toggle 3
        </Toggle>
      </div>
    </div>
  {/each}
</div>

<div>{value}</div>

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each shapes as shape (shape)}
    {#each colors as color (color)}
      {@const title = toTitleCase(shape) + " " + toTitleCase(color)}
      <div>
        <ToggleGroup bind:value>
          {#snippet children(toggleGroup)}
            <ToggleButton {toggleGroup} {shape} {color} value="1">
              {title} 1
            </ToggleButton>
            <ToggleButton {toggleGroup} {shape} {color} value="2">
              {title} 2
            </ToggleButton>
            <ToggleButton {toggleGroup} {shape} {color} value="3">
              {title} 3
            </ToggleButton>
          {/snippet}
        </ToggleGroup>
      </div>
    {/each}
  {/each}
</div>
