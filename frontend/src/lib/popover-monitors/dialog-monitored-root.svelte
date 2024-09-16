<script lang="ts">
  import { Root } from "$lib/components/ui/dialog";
  import { type SvelteComponent } from "svelte";
  import { handleOpenChange } from ".";

  export let open: boolean = false;
  let component: SvelteComponent;
  $: handleOpenChange(open, component);
</script>

<Root {...$$restProps} bind:open bind:this={component}>
  <slot />
</Root>
