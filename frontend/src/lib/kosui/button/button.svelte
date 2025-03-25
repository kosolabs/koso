<script lang="ts" module>
  import type { Icon } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import { Tooltip } from "../tooltip";
  import { noop, type ClassName, type ElementRef } from "../utils";

  export type ButtonProps = {
    icon?: typeof Icon;
    size?: number;
    tooltip?: Snippet | string;
  } & ElementRef &
    ClassName &
    Variants &
    HTMLButtonAttributes;
</script>

<script lang="ts">
  let {
    icon: IconComponent,
    size = 16,
    tooltip,
    children,
    el = $bindable(),
    ref = noop,
    class: className,
    variant = "outlined",
    color = "primary",
    shape = "rounded",
    ...restProps
  }: ButtonProps = $props();

  let tooltipRef: Tooltip | undefined = $state();
</script>

<button
  bind:this={el}
  use:ref
  class={twMerge(
    baseClasses({ variant, color, shape, hover: true, focus: true }),
    "flex items-center gap-2 text-sm text-nowrap transition-all enabled:active:scale-95",
    children ? "px-4 py-1.5" : "p-2",
    className,
  )}
  {...restProps}
  {...tooltipRef?.triggerProps}
>
  {#if IconComponent}
    <IconComponent {size} />
  {/if}
  {@render children?.()}
</button>

{#if tooltip}
  <Tooltip bind:this={tooltipRef} anchorEl={el} arrow>
    {#if typeof tooltip === "function"}
      {@render tooltip()}
    {:else}
      {tooltip}
    {/if}
  </Tooltip>
{/if}
