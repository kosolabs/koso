<script lang="ts" module>
  import { type Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import type { TooltipTriggerProps } from ".";
  import type { PopoverProps } from "../popover";
  import { Popover } from "../popover";
  import type { ClassName } from "../utils";

  type SnippetTooltipTriggerProps = {
    ref: (el: HTMLElement) => void;
  } & Omit<TooltipTriggerProps, "ref">;

  export type TooltipProps = {
    delay?: number;
    open?: boolean;
    trigger?: Snippet<[SnippetTooltipTriggerProps]>;
  } & ClassName &
    PopoverProps;
</script>

<script lang="ts">
  let {
    delay = 1000,
    open = $bindable(false),
    trigger,
    anchorEl,
    class: className,
    ...restProps
  }: TooltipProps = $props();

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

  export const triggerProps: SnippetTooltipTriggerProps = {
    ref: (ref) => (anchorEl = ref),
    onmouseenter: () => show(),
    onmouseleave: () => hide(),
  };
</script>

<Popover
  bind:open
  role="tooltip"
  {anchorEl}
  class={twMerge(
    "bg-m3-inverse-surface text-m3-inverse-on-surface overflow-visible rounded-sm px-2 py-1 text-xs text-wrap",
    className,
  )}
  {...restProps}
/>

{@render trigger?.(triggerProps)}
