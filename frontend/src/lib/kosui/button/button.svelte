<script lang="ts" module>
  import type { Icon } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import { Tooltip } from "../tooltip";
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

  let tooltipRef: Tooltip | undefined = $state();
</script>

<button
  bind:this={ref}
  class={twMerge(
    baseClasses({ variant, color, shape, hover: true, focus: true }),
    "flex items-center gap-2 px-4 py-1.5 text-sm text-nowrap transition-all enabled:active:scale-95",
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
  <Tooltip bind:this={tooltipRef} trigger={ref} arrow>
    {#if typeof tooltip === "function"}
      {@render tooltip()}
    {:else}
      {tooltip}
    {/if}
  </Tooltip>
{/if}
