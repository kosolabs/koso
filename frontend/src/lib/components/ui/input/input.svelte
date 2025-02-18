<script lang="ts">
  import type { ElementRef } from "$lib/kosui/utils";
  import { cn } from "$lib/utils.js";
  import type { HTMLInputAttributes } from "svelte/elements";

  let {
    ref = $bindable(),
    value = $bindable(),
    class: className,
    autofocus,
    ...restProps
  }: ElementRef & HTMLInputAttributes = $props();

  $effect(() => {
    if (ref && autofocus) {
      ref.focus();
    }
  });
</script>

<input
  bind:this={ref}
  class={cn(
    "border-input placeholder:text-muted-foreground focus-visible:ring-ring flex h-9 w-full rounded-md border bg-transparent px-3 py-1 text-sm shadow-xs transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium focus-visible:ring-1 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50",
    className,
  )}
  bind:value
  {...restProps}
/>
