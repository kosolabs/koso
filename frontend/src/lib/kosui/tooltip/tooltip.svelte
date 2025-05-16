<script lang="ts" module>
  import { type Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import { Popover, type PopoverProps } from "../popover";
  import type { ClassName } from "../utils";

  export type TooltipTriggerProps = {
    ref: (el: HTMLElement) => void;
    onclick?: () => void;
    onmouseenter?: () => void;
    onmouseleave?: () => void;
  };

  export type TooltipProps = {
    delay?: number;
    open?: boolean;
    click?: boolean;
    trigger?: Snippet<[TooltipTriggerProps]>;
  } & ClassName &
    PopoverProps;
</script>

<script lang="ts">
  let {
    delay = 1000,
    open = $bindable(false),
    click = false,
    trigger,
    anchorEl,
    class: className,
    ...restProps
  }: TooltipProps = $props();

  let timeout: number;

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

  const triggerProps: TooltipTriggerProps = click
    ? {
        ref: (ref) => (anchorEl = ref),
        onclick: () => (open = true),
      }
    : {
        ref: (ref) => (anchorEl = ref),
        onmouseenter: () => show(delay),
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
