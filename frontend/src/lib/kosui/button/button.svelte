<script lang="ts" module>
  import type { Icon } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { type ClassValue, type VariantProps, tv } from "tailwind-variants";
  import { PlainTooltip } from "../tooltip";
  import type { ElementRef } from "../utils";

  export const buttonVariants = tv({
    base: "rounded-m3 disabled:text-m3-on-surface/38 shadow-m3-shadow/38 flex items-center gap-2 px-4 py-1.5 text-sm text-nowrap transition-all focus-visible:ring-1 focus-visible:outline-hidden not-disabled:active:scale-95",
    variants: {
      variant: {
        elevated:
          "bg-m3-surface-container-low disabled:bg-m3-on-surface/12 not-disabled:not-active:shadow",
        filled: "disabled:bg-m3-on-surface/12",
        tonal: "disabled:bg-m3-on-surface/12",
        outline: "outline-m3-outline disabled:outline-m3-on-surface/12 outline",
        ghost: "disabled:outline-m3-on-surface/12",
      },
      color: {
        primary: "focus-visible:ring-m3-primary",
        secondary: "focus-visible:ring-m3-secondary",
        tertiary: "focus-visible:ring-m3-tertiary",
        error: "focus-visible:ring-m3-error",
      },
    },
    compoundVariants: [
      {
        variant: "elevated",
        color: "primary",
        class:
          "text-m3-primary hover:bg-m3-primary-container/30 focus-visible:bg-m3-primary-container/30",
      },
      {
        variant: "elevated",
        color: "secondary",
        class:
          "text-m3-secondary hover:bg-m3-secondary-container/30 focus-visible:bg-m3-secondary-container/30",
      },
      {
        variant: "elevated",
        color: "tertiary",
        class:
          "text-m3-tertiary hover:bg-m3-tertiary-container/30 focus-visible:bg-m3-tertiary-container/30",
      },
      {
        variant: "elevated",
        color: "error",
        class:
          "text-m3-error hover:bg-m3-error-container/30 focus-visible:bg-m3-error-container/30",
      },
      {
        variant: "filled",
        color: "primary",
        class:
          "bg-m3-primary text-m3-on-primary hover:bg-m3-primary/90 focus-visible:bg-m3-primary/90",
      },
      {
        variant: "filled",
        color: "secondary",
        class:
          "bg-m3-secondary text-m3-on-secondary hover:bg-m3-secondary/90 focus-visible:bg-m3-secondary/90",
      },
      {
        variant: "filled",
        color: "tertiary",
        class:
          "bg-m3-tertiary text-m3-on-tertiary hover:bg-m3-tertiary/90 focus-visible:bg-m3-tertiary/90",
      },
      {
        variant: "filled",
        color: "error",
        class:
          "bg-m3-error text-m3-on-error hover:bg-m3-error/90 focus-visible:bg-m3-error/90",
      },
      {
        variant: "tonal",
        color: "primary",
        class:
          "bg-m3-primary-container text-m3-on-primary-container hover:bg-m3-primary-container/80 focus-visible:bg-m3-primary-container/80",
      },
      {
        variant: "tonal",
        color: "secondary",
        class:
          "bg-m3-secondary-container text-m3-on-secondary-container hover:bg-m3-secondary-container/80 focus-visible:bg-m3-secondary-container/80",
      },
      {
        variant: "tonal",
        color: "tertiary",
        class:
          "bg-m3-tertiary-container text-m3-on-tertiary-container hover:bg-m3-tertiary-container/80 focus-visible:bg-m3-tertiary-container/80",
      },
      {
        variant: "tonal",
        color: "error",
        class:
          "bg-m3-error-container text-m3-on-error-container hover:bg-m3-error-container/80 focus-visible:bg-m3-error-container/80",
      },
      {
        variant: "outline",
        color: "primary",
        class: "focus-visible:outline-m3-primary",
      },
      {
        variant: "outline",
        color: "secondary",
        class: "focus-visible:outline-m3-secondary",
      },
      {
        variant: "outline",
        color: "tertiary",
        class: "focus-visible:outline-m3-tertiary",
      },
      {
        variant: "outline",
        color: "error",
        class: "focus-visible:outline-m3-error",
      },
      {
        variant: ["outline", "ghost"],
        color: "primary",
        class:
          "text-m3-primary not-disabled:hover:bg-m3-primary/15 focus-visible:bg-m3-primary/15",
      },
      {
        variant: ["outline", "ghost"],
        color: "secondary",
        class:
          "text-m3-secondary not-disabled:hover:bg-m3-secondary/15 focus-visible:bg-m3-secondary/15",
      },
      {
        variant: ["outline", "ghost"],
        color: "tertiary",
        class:
          "text-m3-tertiary not-disabled:hover:bg-m3-tertiary/15 focus-visible:bg-m3-tertiary/15",
      },
      {
        variant: ["outline", "ghost"],
        color: "error",
        class:
          "text-m3-error not-disabled:hover:bg-m3-error/15 focus-visible:bg-m3-error/15",
      },
    ],
    defaultVariants: {
      variant: "outline",
      color: "primary",
    },
  });

  export type ButtonVariants = VariantProps<typeof buttonVariants>;

  export type ButtonProps = {
    class?: ClassValue;
    icon?: typeof Icon;
    tooltip?: Snippet | string;
  } & ElementRef &
    HTMLButtonAttributes &
    ButtonVariants;
</script>

<script lang="ts">
  let {
    class: className,
    variant,
    color,
    ref = $bindable(),
    icon: IconComponent,
    children,
    tooltip,
    ...restProps
  }: ButtonProps = $props();

  let tooltipRef: PlainTooltip | undefined = $state();
</script>

<button
  bind:this={ref}
  class={buttonVariants({ variant, color, className })}
  {...restProps}
  {...tooltipRef?.triggerProps}
>
  {#if IconComponent}
    <IconComponent size={16} />
  {/if}
  {@render children?.()}
</button>

{#if tooltip}
  <PlainTooltip bind:this={tooltipRef} trigger={ref} arrow>
    {#if typeof tooltip === "function"}
      {@render tooltip()}
    {:else}
      {tooltip}
    {/if}
  </PlainTooltip>
{/if}
