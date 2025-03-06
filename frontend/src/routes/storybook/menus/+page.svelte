<script>
  import { Button } from "$lib/kosui/button";
  import { Menu, MenuItem } from "$lib/kosui/menu";

  let open = $state(false);
  let el = $state();
</script>

<div class="flex flex-wrap items-center gap-2 rounded-lg border p-4">
  <Button bind:el onclick={() => (open = true)}>Controlled</Button>
  <Menu bind:open anchorEl={el}>
    {#snippet content(menuItemProps)}
      <MenuItem
        onSelect={() => console.log("controlled item 1")}
        {...menuItemProps}
      >
        Item 1
      </MenuItem>
      <MenuItem
        onSelect={() => console.log("controlled item 2")}
        {...menuItemProps}
      >
        Item 2
      </MenuItem>
    {/snippet}
  </Menu>

  <Menu>
    {#snippet trigger(menuTriggerProps)}
      <Button {...menuTriggerProps}>Render Delegated</Button>
    {/snippet}
    {#snippet content(menuItemProps)}
      <MenuItem
        onSelect={() => console.log("delegated item 1")}
        {...menuItemProps}
      >
        Item 1
      </MenuItem>
      <MenuItem
        onSelect={() => console.log("delegated item 2")}
        {...menuItemProps}
      >
        Item 2
      </MenuItem>
      <MenuItem
        onSelect={() => console.log("delegated item 3")}
        {...menuItemProps}
      >
        Item 3
      </MenuItem>
    {/snippet}
  </Menu>
</div>
