<script>
  import { Fab } from "$lib/kosui/fab";
  import { Plus } from "lucide-svelte";
</script>

<div class="flex flex-wrap gap-2">
  <div
    class="bg-m3-surface-container-highest relative h-48 w-96 rounded-lg p-2"
  >
    <Fab icon={Plus} />
  </div>

  <div
    class="bg-m3-surface-container-highest relative h-48 w-96 rounded-lg p-2"
  >
    <Fab icon={Plus}>Add Task</Fab>
  </div>

  <div
    class="bg-m3-surface-container-highest relative h-48 w-96 overflow-scroll rounded-lg p-2"
  >
    {#each [...Array(20).keys()] as i (i)}
      <div class="text-nowrap">
        Very wide content {i} that overflows the div by being very long
      </div>
    {/each}
    <Fab icon={Plus} reserve />
  </div>

  <div
    class="bg-m3-surface-container-highest relative h-48 w-96 overflow-scroll rounded-lg p-2"
  >
    {#each [...Array(20).keys()] as i (i)}
      <div class="text-nowrap">
        Very wide content {i} that overflows the div by being very long
      </div>
    {/each}
    <Fab verticalAlignment="top" icon={Plus} />
  </div>

  <div
    class="bg-m3-surface-container-highest relative h-48 w-96 overflow-scroll rounded-lg p-2"
  >
    {#each [...Array(20).keys()] as i (i)}
      <div class="text-nowrap">
        Very wide content {i} that overflows the div by being very long
      </div>
    {/each}
    <Fab verticalAlignment="top" horizontalAlignment="left" icon={Plus} />
  </div>
</div>
