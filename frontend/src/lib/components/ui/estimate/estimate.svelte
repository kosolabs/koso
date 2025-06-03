<script lang="ts">
  import { Menu, MenuContent, MenuItem, MenuTrigger } from "$lib/kosui/menu";
  import { ESTIMATES, type Estimate } from "$lib/yproxy";
  import { CalendarDays, CircleSlash } from "lucide-svelte";

  type Props = {
    value: number | null;
    editable: boolean;
    onSelect: (select: Estimate | null) => void;
  };
  let { value, editable, onSelect }: Props = $props();

  function select(deadline: Estimate | null) {
    onSelect(deadline);
  }

  function formatEstimate(estimate: number | null): string {
    if (estimate === null) {
      return "Unset";
    }
    if (estimate === 1) {
      return "1 day";
    }
    return estimate + " days";
  }
</script>

{#snippet format(value: number | null)}
  {#if value === null}
    <CircleSlash size={16} />
  {:else}
    {value}
  {/if}
{/snippet}

{#if editable}
  <Menu>
    <MenuTrigger class="px-2 text-sm" title="Task estimate">
      {@render format(value)}
    </MenuTrigger>
    <MenuContent>
      <MenuItem
        class="flex items-center gap-2 rounded text-sm"
        onSelect={() => select(null)}
        title="Unset"
      >
        <CircleSlash size={16} />
        Unset
      </MenuItem>
      {#each ESTIMATES as estimate (estimate)}
        <MenuItem
          class="flex items-center gap-2 rounded text-sm"
          onSelect={() => select(estimate)}
        >
          <CalendarDays size={16} />
          {formatEstimate(estimate)}
        </MenuItem>
      {/each}
    </MenuContent>
  </Menu>
{:else}
  <div class="text-sm" title="Task estimate">
    {@render format(value)}
  </div>
{/if}
