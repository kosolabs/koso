<script lang="ts" module>
  import * as floatingUi from "@floating-ui/dom";
  import { onMount, type Snippet } from "svelte";
  import type {
    ClassValue,
    FocusEventHandler,
    MouseEventHandler,
  } from "svelte/elements";
  import { tv, type VariantProps } from "tailwind-variants";
  import { Box } from "../box.svelte";
  import { cn } from "../utils";

  export const tooltipVariants = tv({
    base: "tooltip-animation bg-m3-inverse-surface text-m3-inverse-on-surface overflow-visible rounded-sm px-2 py-1 text-xs",
  });

  export type TooltipTriggerProps = {
    onfocus?: FocusEventHandler<HTMLElement> | undefined | null;
    onblur?: FocusEventHandler<HTMLElement> | undefined | null;
    onmouseenter?: MouseEventHandler<HTMLElement> | undefined | null;
    onmouseleave?: MouseEventHandler<HTMLElement> | undefined | null;
  };

  export type TooltipVariants = VariantProps<typeof tooltipVariants>;

  export type TooltipProps = TooltipVariants & {
    class?: ClassValue | null;
    delay?: number;
    arrow?: boolean;
    placement?: floatingUi.Placement;
    strategy?: floatingUi.Strategy;
    children: Snippet;
    // If trigger is a Snippet, do render delegation.
    // If trigger is a HTMLElement, do fully controlled.
    trigger?: Snippet<[Box<HTMLElement>, TooltipTriggerProps]> | HTMLElement;
  };
</script>

<script lang="ts">
  let {
    class: className,
    delay = 500,
    arrow = false,
    placement = "top",
    strategy = "fixed",
    children,
    trigger,
    ...restProps
  }: TooltipProps = $props();

  let popoverEl: HTMLDivElement | undefined = $state();
  let arrowEl: HTMLDivElement | undefined = $state();
  let triggerBox = new Box<HTMLElement>();
  let triggerEl = $derived(
    // If trigger is a snippet, get the trigger element from the box.
    // Otherwise, use the passed in element.
    typeof trigger === "function" ? triggerBox.value : trigger,
  );

  let timeout: number | undefined = $state();

  export function show() {
    timeout = window.setTimeout(() => {
      popoverEl?.showPopover();
    }, delay);
  }

  export function hide() {
    popoverEl?.hidePopover();
    window.clearTimeout(timeout);
  }

  export const triggerProps: TooltipTriggerProps = {
    onmouseenter: show,
    onmouseleave: hide,
    onfocus: show,
    onblur: hide,
  };

  async function updatePosition() {
    if (triggerEl && popoverEl && arrowEl) {
      const computedPosition = await floatingUi.computePosition(
        triggerEl,
        popoverEl,
        {
          placement,
          middleware: [
            floatingUi.offset(6),
            floatingUi.flip(),
            floatingUi.shift({ padding: 5 }),
            floatingUi.arrow({ element: arrowEl }),
          ],
          strategy,
        },
      );

      Object.assign(popoverEl.style, {
        left: `${computedPosition.x}px`,
        top: `${computedPosition.y}px`,
      });

      if (computedPosition.middlewareData.arrow) {
        const arrow = computedPosition.middlewareData.arrow;

        const staticSide = {
          top: "bottom",
          right: "left",
          bottom: "top",
          left: "right",
        }[computedPosition.placement.split("-")[0]]!;

        Object.assign(arrowEl.style, {
          left: arrow.x != null ? `${arrow.x}px` : "",
          top: arrow.y != null ? `${arrow.y}px` : "",
          right: "",
          bottom: "",
          [staticSide]: "-4px",
        });
      }
    }
  }

  onMount(() => {
    return floatingUi.autoUpdate(triggerEl!, popoverEl!, updatePosition);
  });
</script>

<div
  bind:this={popoverEl}
  popover="auto"
  role="tooltip"
  class={cn(tooltipVariants({}), className)}
  {...restProps}
>
  {@render children()}
  <div
    bind:this={arrowEl}
    class={cn(
      "bg-m3-inverse-surface absolute -z-10 size-2 rotate-45",
      arrow ? "block" : "hidden",
    )}
  ></div>
</div>

{#if typeof trigger === "function"}
  {@render trigger(triggerBox, triggerProps)}
{/if}

<style>
  .tooltip-animation {
    transition:
      overlay 0.15s allow-discrete,
      display 0.15s allow-discrete;

    animation: close-tooltip 0.15s forwards;
  }

  .tooltip-animation:popover-open {
    animation: open-tooltip 0.15s forwards;
  }

  @keyframes open-tooltip {
    from {
      opacity: 0;
      scale: 0.95;
    }
    to {
      opacity: 1;
      scale: 1;
    }
  }

  @keyframes close-tooltip {
    from {
      opacity: 1;
      scale: 1;
    }
    to {
      opacity: 0;
      scale: 0.95;
    }
  }
</style>
