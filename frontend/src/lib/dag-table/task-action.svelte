<script lang="ts">
  import { baseClasses } from "$lib/kosui/base";
  import { type Action, command } from "$lib/kosui/command";
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

  const indentTask = command.get("Indent task");
  const unindentTask = command.get("Unindent task");
  const subTask = command.get("Insert subtask");
  const deleteTask = command.get("Delete task");

  const moveUp = command.get("Move up");
  const moveDown = command.get("Move down");
  const moveToStart = command.get("Move to start");
  const moveToEnd = command.get("Move to end");

  const linkTask = command.get("Link task to...");
  const blockTask = command.get("Block task on...");
</script>

{#snippet item(action: Action | undefined)}
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
    {@render item(indentTask)}
    {@render item(unindentTask)}
    {@render item(subTask)}
    {@render item(deleteTask)}
    <MenuDivider />
    <MenuHeader>Reorder</MenuHeader>
    {@render item(moveUp)}
    {@render item(moveDown)}
    {@render item(moveToStart)}
    {@render item(moveToEnd)}
    <MenuDivider />
    <MenuHeader>Linking</MenuHeader>
    {@render item(linkTask)}
    {@render item(blockTask)}
  </MenuContent>
</Menu>
