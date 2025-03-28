<script lang="ts">
  import { command, type ActionID } from "$lib/components/ui/command-palette";
  import { baseClasses } from "$lib/kosui/base";
  import {
    Menu,
    MenuContent,
    MenuDivider,
    MenuHeader,
    MenuItem,
    MenuTrigger,
  } from "$lib/kosui/menu";
  import { MoreVertical } from "lucide-svelte";
  import { twMerge } from "tailwind-merge";
</script>

{#snippet item(id: ActionID)}
  {@const action = command.get(id)}
  {#if action}
    <MenuItem
      onSelect={action.callback}
      disabled={!action.enabled()}
      title={action.description}
    >
      {action.title}
      {#if action.shortcut}
        <div class="ml-auto pl-2 text-xs">
          {action.shortcut.toString()}
        </div>
      {/if}
    </MenuItem>
  {/if}
{/snippet}

<Menu>
  <MenuTrigger
    title="Task Actions"
    class={twMerge(
      baseClasses({
        variant: "plain",
        color: "primary",
        shape: "circle",
        focus: true,
        hover: true,
      }),
      "m-0 p-2 transition-all active:scale-95",
    )}
  >
    <MoreVertical size={16} />
  </MenuTrigger>
  <MenuContent>
    <MenuHeader>Actions</MenuHeader>
    {@render item("Indent")}
    {@render item("Undent")}
    {@render item("InsertSubtask")}
    {@render item("Delete")}
    <MenuDivider />
    <MenuHeader>Reorder</MenuHeader>
    {@render item("MoveUp")}
    {@render item("MoveDown")}
    {@render item("MoveToStart")}
    {@render item("MoveToEnd")}
    <MenuDivider />
    <MenuHeader>Linking</MenuHeader>
    {@render item("Link")}
    {@render item("Block")}
  </MenuContent>
</Menu>
