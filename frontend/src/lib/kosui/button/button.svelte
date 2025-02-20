<script lang="ts" module>
  import type { Icon } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { type ClassValue, type VariantProps, tv } from "tailwind-variants";
  import { PlainTooltip } from "../tooltip";
  import type { ElementRef } from "../utils";

  export const buttonVariants = tv({
    base: "rounded-m3 disabled:text-m3-on-surface/38 shadow-m3-shadow/38 flex items-center gap-2 text-sm text-nowrap transition-all focus-visible:ring-1 focus-visible:outline-hidden not-disabled:active:scale-95",
    variants: {
      variant: {
        elevated:
          "bg-m3-surface-container-low disabled:bg-m3-on-surface/12 not-disabled:not-active:shadow",
        filled:
          "disabled:bg-m3-on-surface/12 hover:opacity-90 focus-visible:opacity-90",
        tonal:
          "disabled:bg-m3-on-surface/12 hover:opacity-80 focus-visible:opacity-80",
        outlined:
          "outline-m3-outline disabled:outline-m3-on-surface/12 outline",
        plain: "disabled:outline-m3-on-surface/12",
      },
      color: {
        primary: "focus-visible:ring-m3-primary",
        secondary: "focus-visible:ring-m3-secondary",
        tertiary: "focus-visible:ring-m3-tertiary",
        error: "focus-visible:ring-m3-error",
      },
      scale: {
        sm: "px-3 py-1 text-sm",
        md: "px-4 py-1.5",
        lg: "px-6 py-2 text-lg",
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
      {
        variant: "outlined",
        color: "primary",
        class: "focus-visible:outline-m3-primary",
      },
      {
        variant: "outlined",
        color: "secondary",
        class: "focus-visible:outline-m3-secondary",
      },
      {
        variant: "outlined",
        color: "tertiary",
        class: "focus-visible:outline-m3-tertiary",
      },
      {
        variant: "outlined",
        color: "error",
        class: "focus-visible:outline-m3-error",
      },
      {
        variant: ["outlined", "plain"],
        color: "primary",
        class:
          "text-m3-primary not-disabled:hover:bg-m3-primary/15 focus-visible:bg-m3-primary/15",
      },
      {
        variant: ["outlined", "plain"],
        color: "secondary",
        class:
          "text-m3-secondary not-disabled:hover:bg-m3-secondary/15 focus-visible:bg-m3-secondary/15",
      },
      {
        variant: ["outlined", "plain"],
        color: "tertiary",
        class:
          "text-m3-tertiary not-disabled:hover:bg-m3-tertiary/15 focus-visible:bg-m3-tertiary/15",
      },
      {
        variant: ["outlined", "plain"],
        color: "error",
        class:
          "text-m3-error not-disabled:hover:bg-m3-error/15 focus-visible:bg-m3-error/15",
      },
    ],
    defaultVariants: {
      variant: "outlined",
      color: "primary",
      scale: "md",
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
    scale,
    ref = $bindable(),
    icon: IconComponent,
    children,
    tooltip,
    ...restProps
  }: ButtonProps = $props();

  let tooltipRef: PlainTooltip | undefined = $state();

  let size = $derived.by(() => {
    switch (scale) {
      case "lg":
        return 18;
      case "sm":
        return 14;
      default:
        return 16;
    }
  });
</script>

<button
  bind:this={ref}
  class={buttonVariants({ variant, color, scale, className })}
  {...restProps}
  {...tooltipRef?.triggerProps}
>
  {#if IconComponent}
    <IconComponent {size} />
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
