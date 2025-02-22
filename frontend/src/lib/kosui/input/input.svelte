<script lang="ts" module>
  import type { HTMLInputAttributes } from "svelte/elements";
  import { tv, type VariantProps } from "tailwind-variants";
  import { baseVariants } from "../base";
  import type { ClassName, ElementRef } from "../utils";

  export const inputVariants = tv({
    extend: baseVariants,
    base: "bg-m3-surface-container",
    variants: {
      scale: {
        sm: "h-8 px-2",
        md: "h-9 px-3",
        lg: "h-11 px-4",
      },
    },
    defaultVariants: {
      variant: "outlined",
      color: "secondary",
      scale: "md",
    },
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
    variant,
    color,
    scale,
    ...restProps
  }: InputProps = $props();
</script>

<input
  bind:this={ref}
  bind:value
  class={inputVariants({ variant, color, scale, className })}
  {...restProps}
/>
