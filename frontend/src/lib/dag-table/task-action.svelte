<script lang="ts">
  import { command, type ActionID } from "$lib/components/ui/command-palette";
  import { baseClasses } from "$lib/kosui/base";
  import { Action } from "$lib/kosui/command";
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

  function getActions(ids: ActionID[]): Action<ActionID>[] {
    const actions = [];
    for (const id of ids) {
      const action = command.get(id);
      if (action) {
        actions.push(action);
      }
    }
    return actions;
  }
</script>

{#snippet item(action: Action<ActionID>)}
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
{/snippet}

{#snippet items(actions: Action<ActionID>[])}
  {#each actions as action (action.id)}
    {@render item(action)}
  {/each}
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
    {@const actions = getActions([
      "Indent",
      "Undent",
      "InsertSubtask",
      "Delete",
    ])}

    {#if actions.length > 0}
      <MenuHeader>Actions</MenuHeader>
      {@render items(actions)}
      <MenuDivider />
    {/if}

    {@const reorder = getActions([
      "MoveUp",
      "MoveDown",
      "MoveToStart",
      "MoveToEnd",
    ])}
    {#if reorder.length > 0}
      <MenuHeader>Reorder</MenuHeader>
      {@render items(reorder)}
      <MenuDivider />
    {/if}

    {@const linking = getActions(["Link", "Block"])}
    {#if linking.length > 0}
      <MenuHeader>Linking</MenuHeader>
      {@render items(linking)}
    {/if}
  </MenuContent>
</Menu>
