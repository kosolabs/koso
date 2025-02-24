<script lang="ts" module>
  import type { Icon } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { PlainTooltip } from "../tooltip";
  import type { ClassName, ElementRef } from "../utils";

  export type ButtonVariant =
    | "elevated"
    | "filled"
    | "tonal"
    | "outlined"
    | "plain";
  export type ButtonColor = "primary" | "secondary" | "tertiary" | "error";

  export type ButtonVariants = {
    variant?: ButtonVariant;
    color?: ButtonColor;
  };

  export type ButtonProps = {
    icon?: typeof Icon;
    tooltip?: Snippet | string;
    variant?: ButtonVariant;
    color?: ButtonColor;
  } & ElementRef &
    ClassName &
    ButtonVariants &
    HTMLButtonAttributes;
</script>

<script lang="ts">
  let {
    icon: IconComponent,
    children,
    tooltip,
    ref = $bindable(),
    class: className,
    variant = "outlined",
    color = "primary",
    ...restProps
  }: ButtonProps = $props();

  let tooltipRef: PlainTooltip | undefined = $state();
</script>

<button
  bind:this={ref}
  class={twMerge(
    "disabled:text-m3-on-surface/38 shadow-m3-shadow/20 disabled:bg-m3-on-surface/12 rounded-m3 flex items-center gap-2 px-4 py-1.5 text-sm text-nowrap transition-all focus-visible:ring-1 focus-visible:outline-hidden not-disabled:active:scale-95 disabled:cursor-not-allowed",
    variant === "elevated" &&
      "bg-m3-surface-container-low not-disabled:not-active:shadow",
    variant === "filled" && "hover:opacity-90 focus-visible:opacity-90",
    variant === "tonal" && "hover:opacity-80 focus-visible:opacity-80",
    variant === "outlined" && "ring-1 focus-visible:ring-2",
    variant === "plain" && "",

    color === "primary" && "text-m3-primary focus-visible:ring-m3-primary",
    color === "secondary" &&
      "text-m3-secondary focus-visible:ring-m3-secondary",
    color === "tertiary" && "focus-visible:ring-m3-tertiary text-m3-tertiary",
    color === "error" && "focus-visible:ring-m3-error text-m3-error",

    variant === "elevated" &&
      color === "primary" &&
      "hover:bg-m3-primary-container/30 focus-visible:bg-m3-primary-container/30",
    variant === "elevated" &&
      color === "secondary" &&
      "hover:bg-m3-secondary-container/30 focus-visible:bg-m3-secondary-container/30",
    variant === "elevated" &&
      color === "tertiary" &&
      "hover:bg-m3-tertiary-container/30 focus-visible:bg-m3-tertiary-container/30",
    variant === "elevated" &&
      color === "error" &&
      "hover:bg-m3-error-container/30 focus-visible:bg-m3-error-container/30",

    variant === "filled" &&
      color === "primary" &&
      "bg-m3-primary text-m3-on-primary",
    variant === "filled" &&
      color === "secondary" &&
      "bg-m3-secondary text-m3-on-secondary",
    variant === "filled" &&
      color === "tertiary" &&
      "bg-m3-tertiary text-m3-on-tertiary",
    variant === "filled" && color === "error" && "bg-m3-error text-m3-on-error",

    variant === "tonal" &&
      color === "primary" &&
      "bg-m3-primary-container text-m3-on-primary-container",
    variant === "tonal" &&
      color === "secondary" &&
      "bg-m3-secondary-container text-m3-on-secondary-container",
    variant === "tonal" &&
      color === "tertiary" &&
      "bg-m3-tertiary-container text-m3-on-tertiary-container",
    variant === "tonal" &&
      color === "error" &&
      "bg-m3-error-container text-m3-on-error-container",

    variant === "outlined" &&
      color === "primary" &&
      "not-disabled:hover:bg-m3-primary/15 focus-visible:bg-m3-primary/15",
    variant === "outlined" &&
      color === "secondary" &&
      "not-disabled:hover:bg-m3-secondary/15 focus-visible:bg-m3-secondary/15",
    variant === "outlined" &&
      color === "tertiary" &&
      "not-disabled:hover:bg-m3-tertiary/15 focus-visible:bg-m3-tertiary/15",
    variant === "outlined" &&
      color === "error" &&
      "not-disabled:hover:bg-m3-error/15 focus-visible:bg-m3-error/15",

    variant === "plain" &&
      color === "primary" &&
      "not-disabled:hover:bg-m3-primary/15 focus-visible:bg-m3-primary/15",
    variant === "plain" &&
      color === "secondary" &&
      "not-disabled:hover:bg-m3-secondary/15 focus-visible:bg-m3-secondary/15",
    variant === "plain" &&
      color === "tertiary" &&
      "not-disabled:hover:bg-m3-tertiary/15 focus-visible:bg-m3-tertiary/15",
    variant === "plain" &&
      color === "error" &&
      "not-disabled:hover:bg-m3-error/15 focus-visible:bg-m3-error/15",
    className,
  )}
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
