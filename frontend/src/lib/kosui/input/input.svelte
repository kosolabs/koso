<script lang="ts" module>
  import type { HTMLInputAttributes } from "svelte/elements";
  import { tv, type VariantProps } from "tailwind-variants";
  import type { ClassName, ElementRef } from "../utils";

  export const inputVariants = tv({
    base: "disabled:text-m3-on-surface/38 disabled:bg-m3-on-surface/12 rounded-m3 shadow-xs focus-visible:ring-2 focus-visible:outline-hidden disabled:cursor-not-allowed",
    variants: {
      variant: {
        elevated: "bg-m3-surface-container not-disabled:not-active:shadow",
        filled: "",
        tonal: "",
        outlined: "bg-m3-surface-container ring-1",
        plain: "bg-m3-surface-container",
      },
      color: {
        primary: "text-m3-primary focus-visible:ring-m3-primary",
        secondary: "text-m3-secondary focus-visible:ring-m3-secondary",
        tertiary: "text-m3-tertiary focus-visible:ring-m3-tertiary",
        error: "text-m3-error focus-visible:ring-m3-error",
      },
      scale: {
        sm: "h-8 px-2 text-sm",
        md: "h-9 px-3",
        lg: "h-11 px-4 text-lg",
      },
    },
    compoundVariants: [
      {
        variant: "filled",
        color: "primary",
        class: "bg-m3-primary text-m3-on-primary",
      },
      {
        variant: "filled",
        color: "secondary",
        class: "bg-m3-secondary text-m3-on-secondary",
      },
      {
        variant: "filled",
        color: "tertiary",
        class: "bg-m3-tertiary text-m3-on-tertiary",
      },
      {
        variant: "filled",
        color: "error",
        class: "bg-m3-error text-m3-on-error",
      },
      {
        variant: "tonal",
        color: "primary",
        class: "bg-m3-primary-container text-m3-on-primary-container",
      },
      {
        variant: "tonal",
        color: "secondary",
        class: "bg-m3-secondary-container text-m3-on-secondary-container",
      },
      {
        variant: "tonal",
        color: "tertiary",
        class: "bg-m3-tertiary-container text-m3-on-tertiary-container",
      },
      {
        variant: "tonal",
        color: "error",
        class: "bg-m3-error-container text-m3-on-error-container",
      },
    ],
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
