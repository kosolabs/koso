<script lang="ts" module>
  import type { Icon } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import { PlainTooltip } from "../tooltip";
  import type { ClassName, ElementRef } from "../utils";

  export type ButtonProps = {
    icon?: typeof Icon;
    tooltip?: Snippet | string;
  } & ElementRef &
    ClassName &
    Variants &
    HTMLButtonAttributes;
</script>

<script lang="ts">
  let {
    icon: IconComponent,
    tooltip,
    children,
    ref = $bindable(),
    class: className,
    variant = "outlined",
    color = "primary",
    shape = "rounded",
    ...restProps
  }: ButtonProps = $props();

  let tooltipRef: PlainTooltip | undefined = $state();
</script>

<button
  bind:this={ref}
  class={twMerge(
    baseClasses({ variant, color, shape }),

    "flex items-center gap-2 px-4 py-1.5 text-sm text-nowrap transition-all enabled:active:scale-95",

    variant === "elevated" &&
      color === "primary" &&
      "enabled:hover:bg-m3-primary-container/30 focus-visible:bg-m3-primary-container/30",
    variant === "elevated" &&
      color === "secondary" &&
      "enabled:hover:bg-m3-secondary-container/30 focus-visible:bg-m3-secondary-container/30",
    variant === "elevated" &&
      color === "tertiary" &&
      "enabled:hover:bg-m3-tertiary-container/30 focus-visible:bg-m3-tertiary-container/30",
    variant === "elevated" &&
      color === "error" &&
      "enabled:hover:bg-m3-error-container/30 focus-visible:bg-m3-error-container/30",

    (variant === "outlined" || variant === "plain") &&
      color === "primary" &&
      "enabled:hover:bg-m3-primary/15 focus-visible:bg-m3-primary/15",
    (variant === "outlined" || variant === "plain") &&
      color === "secondary" &&
      "enabled:hover:bg-m3-secondary/15 focus-visible:bg-m3-secondary/15",
    (variant === "outlined" || variant === "plain") &&
      color === "tertiary" &&
      "enabled:hover:bg-m3-tertiary/15 focus-visible:bg-m3-tertiary/15",
    (variant === "outlined" || variant === "plain") &&
      color === "error" &&
      "enabled:hover:bg-m3-error/15 focus-visible:bg-m3-error/15",
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
