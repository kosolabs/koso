<script lang="ts">
  import { colors } from "$lib/kosui/base";
  import { Toggle } from "$lib/kosui/toggle";
  import { toTitleCase } from "$lib/kosui/utils";

  let value: "1" | "2" | "3" | undefined = $state();
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
