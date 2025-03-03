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
    useRef = noop,
    class: className,
    variant = "outlined",
    color = "primary",
    shape = "rounded",
    ...restProps
  }: ButtonProps = $props();

  let tooltipRef: Tooltip | undefined = $state();

  $effect(() => {
    if (ref && useRef) {
      useRef(ref);
    }
  });
</script>

<button
  bind:this={ref}
  use:useRef
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
  <Tooltip bind:this={tooltipRef} anchorEl={ref} arrow>
    {#if typeof tooltip === "function"}
      {@render tooltip()}
    {:else}
      {tooltip}
    {/if}
  </Tooltip>
{/if}
