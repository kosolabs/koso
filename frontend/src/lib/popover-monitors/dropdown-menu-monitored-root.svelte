<script lang="ts">
  import { Root } from "$lib/components/ui/dropdown-menu";
  import { type SvelteComponent } from "svelte";
  import { handleOpenChange } from ".";

  export let closeFocus: HTMLElement | undefined = undefined;

  let open: boolean;
  let component: SvelteComponent;
  $: handleOpenChange(open, component);
</script>

<Root {closeFocus} {...$$restProps} bind:open bind:this={component}>
  <slot />
</Root>
