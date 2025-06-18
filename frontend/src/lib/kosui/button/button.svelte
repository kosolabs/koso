<script lang="ts" module>
  import type { Icon } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import { mergeProps } from "../merge-props";
  import { Tooltip, type TooltipTriggerProps } from "../tooltip";
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
</script>

{#snippet button({ ref: anchorRef, ...triggerProps }: TooltipTriggerProps)}
  <button
    bind:this={el}
    use:ref
    use:anchorRef
    class={twMerge(
      baseClasses({ variant, color, shape, hover: true, focus: true }),
      "flex items-center justify-center gap-2 text-sm text-nowrap transition-all select-none enabled:active:scale-95",
      children ? "px-4 py-2" : "p-2",
      className,
    )}
    {...mergeProps(restProps, triggerProps)}
  >
    {#if IconComponent}
      <IconComponent {size} />
    {/if}
    {@render children?.()}
  </button>
{/snippet}

{#if tooltip}
  <Tooltip arrow>
    {#snippet trigger(props)}
      {@render button(props)}
    {/snippet}
    {#if typeof tooltip === "function"}
      {@render tooltip()}
    {:else}
      {tooltip}
    {/if}
  </Tooltip>
{:else}
  {@render button({ ref: noop })}
{/if}
