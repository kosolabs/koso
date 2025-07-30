<script lang="ts">
  import { ESTIMATES, type Estimate } from "$lib/yproxy";
  import { CircleSlash } from "@lucide/svelte";
  import { Menu, MenuContent, MenuItem, MenuTrigger } from "kosui";

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
      return "1 point";
    }
    return estimate + " points";
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
    <MenuTrigger class="text-m3-primary px-2 text-sm" title="Task estimate">
      {@render format(value)}
    </MenuTrigger>
    <MenuContent>
      <MenuItem
        class="flex items-center gap-2 rounded text-sm"
        onSelect={() => select(null)}
        title="Unset"
      >
        Unset
      </MenuItem>
      {#each ESTIMATES as estimate (estimate)}
        <MenuItem
          class="flex items-center gap-2 rounded text-sm"
          onSelect={() => select(estimate)}
        >
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
