<script lang="ts">
  import { Check } from "@lucide/svelte";
  import { MenuDivider, MenuHeader, MenuItem } from ".";
  import type { Action } from "../command";

  export type MenuActionsProps = {
    actions: Action[];
  };
  let { actions }: MenuActionsProps = $props();

  let categoryHasSelected = $derived(
    actions.reduce(
      (acc, { category, selected }) => ({
        ...acc,
        [category]: acc[category] || selected !== undefined,
      }),
      {} as Record<string, boolean>,
    ),
  );
</script>

{#each actions as action, index (action)}
  {@const Icon = action.icon}
  {#if index === 0 || actions[index - 1].category !== action.category}
    {#if index !== 0}
      <MenuDivider />
    {/if}
    <MenuHeader>{action.category}</MenuHeader>
  {/if}
  <MenuItem
    class="gap-2"
    onSelect={action.callback}
    title={action.description}
    disabled={!action.enabled()}
  >
    {#if categoryHasSelected[action.category]}
      {#if action.selected?.()}
        <Check class="text-m3-primary" size={16} />
      {:else}
        <div class="size-4"></div>
      {/if}
    {/if}
    <Icon size={16} />
    {action.name}
    {#if action.shortcut}
      <div class="ml-auto pl-2 text-xs">
        {action.shortcut.toString()}
      </div>
    {/if}
  </MenuItem>
{/each}
