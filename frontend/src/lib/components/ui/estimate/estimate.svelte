<script lang="ts">
  import { Menu, MenuContent, MenuItem, MenuTrigger } from "$lib/kosui/menu";
  import { ESTIMATES, type Estimate } from "$lib/yproxy";
  import ResponsiveText from "../responsive-text/responsive-text.svelte";

  type Props = {
    value: Estimate | null;
    editable?: boolean;
    onSelect?: (select: Estimate | null) => void;
  };
  let { value = null, editable = true, onSelect }: Props = $props();

  function select(deadline: Estimate | null) {
    value = deadline;
    onSelect?.(deadline);
  }

  let open: boolean = $state(false);

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

{#if editable}
  <Menu bind:open>
    <MenuTrigger class="flex items-center gap-2" title={`${value ?? "Unset"}`}>
      <ResponsiveText>{formatEstimate(value)}</ResponsiveText>
    </MenuTrigger>
    <MenuContent>
      <MenuItem onSelect={() => select(null)}>Unset</MenuItem>
      {#each ESTIMATES as estimate (estimate)}
        <MenuItem onSelect={() => select(estimate)}>
          <ResponsiveText>{formatEstimate(estimate)}</ResponsiveText>
        </MenuItem>
      {/each}
    </MenuContent>
  </Menu>
{:else}
  <div class="flex items-center gap-2" title={`${value ?? "Unset"}`}>
    <ResponsiveText>{formatEstimate(value)}</ResponsiveText>
  </div>
{/if}
