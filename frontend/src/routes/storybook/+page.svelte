<script lang="ts">
  import { Goto } from "$lib/kosui/goto";
  import { toTitleCase } from "$lib/kosui/utils";
  import { paths } from "./+layout.svelte";
</script>

<div class="flex flex-col items-start rounded-lg border p-4">
  {#each paths as path (path)}
    <Goto href={path}>
      {toTitleCase(path.split("/").slice(-1)[0])}
    </Goto>
  {/each}
</div>
