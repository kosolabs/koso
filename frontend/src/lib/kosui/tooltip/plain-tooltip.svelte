<script lang="ts" module>
  import { type Snippet } from "svelte";
  import type { MouseEventHandler } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { Box } from "../box.svelte";
  import type { PopoverProps } from "../popover";
  import { Popover } from "../popover";
  import type { ClassName } from "../utils";

  export type TooltipTriggerProps = {
    onmouseenter?: MouseEventHandler<HTMLElement> | undefined | null;
    onmouseleave?: MouseEventHandler<HTMLElement> | undefined | null;
  };

  export type TooltipProps = {
    delay?: number;
    open?: boolean;
    // If trigger is a Snippet, do render delegation.
    // If trigger is a HTMLElement, do fully controlled.
    trigger?: Snippet<[Box<HTMLElement>, TooltipTriggerProps]> | HTMLElement;
    children: Snippet;
  } & ClassName &
    PopoverProps;
</script>

<script lang="ts">
  let {
    delay = 1000,
    open = $bindable(false),
    trigger,
    children,
    class: className,
    ...restProps
  }: TooltipProps = $props();

  let triggerBox = new Box<HTMLElement>();
  let triggerEl = $derived(
    // If trigger is a snippet, get the trigger element from the box.
    // Otherwise, use the passed in element.
    typeof trigger === "function" ? triggerBox.value : trigger,
  );

  let tooltipTimeout: number;

  export function show(after?: number) {
    tooltipTimeout = window.setTimeout(
      () => (open = true),
      after === undefined ? delay : after,
    );
  }

  export function hide() {
    window.clearTimeout(tooltipTimeout);
    open = false;
  }

  export const triggerProps: TooltipTriggerProps = {
    onmouseenter: () => show(),
    onmouseleave: () => hide(),
  };
</script>

<Popover
  bind:open
  role="tooltip"
  anchorEl={triggerEl}
  class={twMerge(
    "bg-m3-inverse-surface text-m3-inverse-on-surface overflow-visible rounded-sm px-2 py-1 text-xs",
    className,
  )}
  {...restProps}
>
  {@render children()}
</Popover>

{#if typeof trigger === "function"}
  {@render trigger(triggerBox, triggerProps)}
{/if}
