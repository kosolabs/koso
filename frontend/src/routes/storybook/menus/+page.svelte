<script>
  import { Button } from "$lib/kosui/button";
  import { Menu, MenuItem } from "$lib/kosui/menu";

  let open = $state(false);
  let el = $state();
</script>

<div class="flex flex-wrap items-center gap-2 rounded-lg border p-4">
  <Button bind:el onclick={() => (open = true)}>Controlled</Button>
  <Menu bind:open anchorEl={el}>
    {#snippet content(menuRef)}
      <MenuItem {menuRef} onSelect={() => console.log("controlled item 1")}>
        Item 1
      </MenuItem>
      <MenuItem {menuRef} onSelect={() => console.log("controlled item 2")}>
        Item 2
      </MenuItem>
    {/snippet}
  </Menu>

  <Menu>
    {#snippet trigger(props)}
      <Button {...props}>Render Delegated</Button>
    {/snippet}
    {#snippet content(menuRef)}
      <MenuItem {menuRef} onSelect={() => console.log("delegated item 1")}>
        Item 1
      </MenuItem>
      <MenuItem {menuRef} onSelect={() => console.log("delegated item 2")}>
        Item 2
      </MenuItem>
    {/snippet}
  </Menu>
</div>
