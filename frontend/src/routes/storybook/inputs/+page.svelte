<script lang="ts">
  import { Input } from "$lib/kosui/input";

  let value: string = $state("");
</script>

<div class="flex flex-wrap items-center gap-2 rounded-lg border p-4">
  <Input
    bind:value
    type="text"
    placeholder="Search"
    name="search"
    autocomplete="off"
  />
</div>
