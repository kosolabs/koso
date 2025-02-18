<script lang="ts" module>
  import { type Snippet } from "svelte";
  import type { FocusEventHandler, MouseEventHandler } from "svelte/elements";
  import { tv, type ClassValue, type VariantProps } from "tailwind-variants";
  import { Box } from "../box.svelte";
  import type { PopoverProps } from "../popover";
  import { Popover } from "../popover";

  export const tooltipVariants = tv({
    base: "bg-m3-inverse-surface text-m3-inverse-on-surface overflow-visible rounded-sm px-2 py-1 text-xs",
  });

  export type TooltipTriggerProps = {
    onfocus?: FocusEventHandler<HTMLElement> | undefined | null;
    onblur?: FocusEventHandler<HTMLElement> | undefined | null;
    onmouseenter?: MouseEventHandler<HTMLElement> | undefined | null;
    onmouseleave?: MouseEventHandler<HTMLElement> | undefined | null;
  };

  export type TooltipVariants = VariantProps<typeof tooltipVariants>;

  export type TooltipProps = TooltipVariants & {
    class?: ClassValue;
    delay?: number;
    open?: boolean;
    children: Snippet;
    // If trigger is a Snippet, do render delegation.
    // If trigger is a HTMLElement, do fully controlled.
    trigger?: Snippet<[Box<HTMLElement>, TooltipTriggerProps]> | HTMLElement;
  } & TooltipVariants &
    PopoverProps;
</script>

<script lang="ts">
  let {
    class: className,
    delay = 500,
    open = $bindable(false),
    children,
    trigger,
    ...restProps
  }: TooltipProps = $props();

  let triggerBox = new Box<HTMLElement>();
  let triggerEl = $derived(
    // If trigger is a snippet, get the trigger element from the box.
    // Otherwise, use the passed in element.
    typeof trigger === "function" ? triggerBox.value : trigger,
  );

  let timeout: number | undefined = $state();

  export function show(after?: number) {
    timeout = window.setTimeout(
      () => (open = true),
      after === undefined ? delay : after,
    );
  }

  export function hide() {
    window.clearTimeout(timeout);
    open = false;
  }

  export const triggerProps: TooltipTriggerProps = {
    onmouseenter: () => show(),
    onmouseleave: () => hide(),
    onfocus: () => show(),
    onblur: () => hide(),
  };
</script>

<Popover
  bind:open
  role="tooltip"
  anchorEl={triggerEl}
  class={tooltipVariants({ className })}
  {...restProps}
>
  {@render children()}
</Popover>

{#if typeof trigger === "function"}
  {@render trigger(triggerBox, triggerProps)}
{/if}
