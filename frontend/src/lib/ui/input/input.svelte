<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLInputAttributes } from "svelte/elements";

  type Props = HTMLInputAttributes;

  let {
    class: classes,
    value = $bindable(),
    autofocus,
    ...props
  }: Props = $props();

  let el: HTMLInputElement;
  $effect(() => {
    if (autofocus) el.focus();
  });
</script>

<input
  class={cn(
    "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
    classes,
  )}
  bind:value
  bind:this={el}
  {...props}
/>
