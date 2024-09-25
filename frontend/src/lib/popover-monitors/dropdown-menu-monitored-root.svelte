<script lang="ts">
  import { Root } from "$lib/components/ui/dropdown-menu";
  import { type SvelteComponent } from "svelte";
  import { handleOpenChange } from ".";

  export let closeFocus: HTMLElement;

  let open: boolean;
  let component: SvelteComponent;
  $: handleOpenChange(open, component);
</script>

<Root {...$$restProps} bind:open bind:this={component} {closeFocus}>
  <slot />
</Root>
