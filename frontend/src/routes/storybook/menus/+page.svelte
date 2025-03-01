<script>
  import { Button } from "$lib/kosui/button";
  import { Menu, MenuItem } from "$lib/kosui/menu";

  let open = $state(false);
  let ref = $state();
</script>

<div class="flex flex-wrap items-center gap-2 rounded-lg border p-4">
  <Button bind:ref onclick={() => (open = true)}>Select</Button>
  <Menu bind:open anchorEl={ref}>
    {#snippet content(menuRef)}
      <MenuItem {menuRef} onSelect={() => console.log("item 1")}>
        Item 1
      </MenuItem>
      <MenuItem {menuRef} onSelect={() => console.log("item 1")}>
        Item 2
      </MenuItem>
    {/snippet}
  </Menu>
</div>
