<script lang="ts" module>
  import type { Icon } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { type ClassValue, type VariantProps, tv } from "tailwind-variants";
  import { baseVariants } from "../base";
  import { PlainTooltip } from "../tooltip";
  import type { ElementRef } from "../utils";

  export const buttonVariants = tv({
    extend: baseVariants,
    base: "flex items-center gap-2 text-nowrap transition-all not-disabled:active:scale-95",
    variants: {
      variant: {
        elevated: "bg-m3-surface-container-low",
        filled: "hover:opacity-90 focus-visible:opacity-90",
        tonal: "hover:opacity-80 focus-visible:opacity-80",
        outlined: "",
        plain: "",
      },
      color: {
        primary: "",
        secondary: "",
        tertiary: "",
        error: "",
      },
      scale: {
        sm: "px-3 py-1",
        md: "px-4 py-1.5 text-sm",
        lg: "px-6 py-2",
      },
    },
    compoundVariants: [
      {
        variant: "elevated",
        color: "primary",
        class:
          "hover:bg-m3-primary-container/30 focus-visible:bg-m3-primary-container/30",
      },
      {
        variant: "elevated",
        color: "secondary",
        class:
          "hover:bg-m3-secondary-container/30 focus-visible:bg-m3-secondary-container/30",
      },
      {
        variant: "elevated",
        color: "tertiary",
        class:
          "hover:bg-m3-tertiary-container/30 focus-visible:bg-m3-tertiary-container/30",
      },
      {
        variant: "elevated",
        color: "error",
        class:
          "hover:bg-m3-error-container/30 focus-visible:bg-m3-error-container/30",
      },
      {
        variant: ["outlined", "plain"],
        color: "primary",
        class:
          "not-disabled:hover:bg-m3-primary/15 focus-visible:bg-m3-primary/15",
      },
      {
        variant: ["outlined", "plain"],
        color: "secondary",
        class:
          "not-disabled:hover:bg-m3-secondary/15 focus-visible:bg-m3-secondary/15",
      },
      {
        variant: ["outlined", "plain"],
        color: "tertiary",
        class:
          "not-disabled:hover:bg-m3-tertiary/15 focus-visible:bg-m3-tertiary/15",
      },
      {
        variant: ["outlined", "plain"],
        color: "error",
        class: "not-disabled:hover:bg-m3-error/15 focus-visible:bg-m3-error/15",
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
