<script>
  import { baseClasses } from "$lib/kosui/base";
  import { Button } from "$lib/kosui/button";
  import {
    Menu,
    MenuContent,
    MenuDivider,
    MenuHeader,
    MenuItem,
    MenuTrigger,
  } from "$lib/kosui/menu";
  import { twMerge } from "tailwind-merge";

  let open = $state(false);
  let el = $state();
</script>

<div class="flex flex-wrap items-center gap-2 rounded-lg border p-4">
  <Button bind:el onclick={() => (open = !open)}>Controlled</Button>
  <Menu bind:open {el}>
    <MenuContent>
      <MenuItem onSelect={() => console.log("controlled item 1")}>
        Item 1
      </MenuItem>
      <MenuItem onSelect={() => console.log("controlled item 2")}>
        Item 2
      </MenuItem>
    </MenuContent>
  </Menu>

  <Menu>
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
      )}>Open Menu</MenuTrigger
    >
    <MenuContent>
      <MenuItem onSelect={() => console.log("delegated item 1")}>
        1st Item
      </MenuItem>
      <MenuItem onSelect={() => console.log("delegated item 2")}>
        2nd Item
      </MenuItem>
      <MenuDivider />
      <MenuHeader>Header</MenuHeader>
      <MenuItem onSelect={() => console.log("delegated item 3")}>
        3rd Item
      </MenuItem>
    </MenuContent>
  </Menu>
</div>
