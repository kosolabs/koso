<script>
  import { baseClasses } from "$lib/kosui/base";
  import { Button } from "$lib/kosui/button";
  import { Menu, MenuItem, MenuTrigger } from "$lib/kosui/menu";
  import { twMerge } from "tailwind-merge";

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
      <MenuTrigger
        class={twMerge(
          baseClasses({
            variant: "outlined",
            color: "primary",
            shape: "rounded",
            hover: true,
            focus: true,
          }),
          "px-4 py-1.5 text-sm transition-all enabled:active:scale-95",
        )}
        {...menuTriggerProps}>Open Menu</MenuTrigger
      >
    {/snippet}
    {#snippet content(menuItemProps)}
      <MenuItem
        onSelect={() => console.log("delegated item 1")}
        {...menuItemProps}
      >
        1st Item
      </MenuItem>
      <MenuItem
        onSelect={() => console.log("delegated item 2")}
        {...menuItemProps}
      >
        2nd Item
      </MenuItem>
      <MenuItem
        onSelect={() => console.log("delegated item 3")}
        {...menuItemProps}
      >
        3rd Item
      </MenuItem>
    {/snippet}
  </Menu>
</div>
