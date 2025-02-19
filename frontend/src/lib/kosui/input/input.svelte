<script lang="ts" module>
  import type { HTMLInputAttributes } from "svelte/elements";
  import { tv, type VariantProps } from "tailwind-variants";
  import type { ClassName, ElementRef } from "../utils";

  export const inputVariants = tv({
    base: "bg-m3-surface-container text-m3-on-surface disabled:text-m3-on-surface/4 focus-visible:ring-m3-primary h-14 rounded px-4 shadow-xs focus-visible:ring-1 focus-visible:outline-hidden disabled:cursor-not-allowed",
  });

  export type InputVariants = VariantProps<typeof inputVariants>;

  export type InputProps = ClassName &
    InputVariants &
    ElementRef &
    HTMLInputAttributes;
</script>

<script lang="ts">
  let {
    ref = $bindable(),
    value = $bindable(),
    class: className,
    ...restProps
  }: InputProps = $props();
</script>

<input
  bind:this={ref}
  bind:value
  class={inputVariants({ className })}
  {...restProps}
/>
