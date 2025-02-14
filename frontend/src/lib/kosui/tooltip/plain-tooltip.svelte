<script lang="ts" module>
  import * as floatingUi from "@floating-ui/dom";
  import { onMount, type Snippet } from "svelte";
  import type {
    ClassValue,
    FocusEventHandler,
    MouseEventHandler,
  } from "svelte/elements";
  import { tv, type VariantProps } from "tailwind-variants";
  import { cn } from "../utils";

  export const tooltipVariants = tv({
    base: "tooltip bg-m3-inverse-surface text-m3-inverse-on-surface overflow-visible rounded-sm px-2 py-1 text-xs",
  });

  export type TooltipTriggerProps = {
    onfocus?: FocusEventHandler<HTMLButtonElement> | undefined | null;
    onblur?: FocusEventHandler<HTMLButtonElement> | undefined | null;
    onmouseenter?: MouseEventHandler<HTMLButtonElement> | undefined | null;
    onmouseleave?: MouseEventHandler<HTMLButtonElement> | undefined | null;
  };

  export type TooltipVariants = VariantProps<typeof tooltipVariants>;

  export type TooltipProps = TooltipVariants & {
    class?: ClassValue | null;
    delay?: number;
    arrow?: boolean;
    placement?: floatingUi.Placement;
    strategy?: floatingUi.Strategy;
    children: Snippet;
    trigger: Snippet<
      [
        TooltipTriggerProps & {
          ref: Box<HTMLButtonElement>;
        },
      ]
    >;
  };
</script>

<script lang="ts">
  import { Box } from "../box.svelte";

  let {
    class: className,
    delay = 500,
    arrow = false,
    children,
    placement = "top",
    strategy = "absolute",
    trigger,
    ...restProps
  }: TooltipProps = $props();

  let popoverEl: HTMLDivElement | undefined = $state();
  let buttonEl: Box<HTMLButtonElement> = new Box();
  let arrowEl: HTMLDivElement | undefined = $state();

  let timeout: number | undefined = $state();

  function handleShow() {
    timeout = window.setTimeout(() => {
      popoverEl?.showPopover();
    }, delay);
  }

  function handleHide() {
    popoverEl?.hidePopover();
    window.clearTimeout(timeout);
  }

  async function updatePosition() {
    if (buttonEl.value && popoverEl && arrowEl) {
      const computedPosition = await floatingUi.computePosition(
        buttonEl.value,
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
    if (buttonEl.value && popoverEl) {
      return floatingUi.autoUpdate(buttonEl.value, popoverEl, updatePosition);
    }
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
      "bg-m3-inverse-surface absolute size-2 rotate-45",
      arrow ? "block" : "hidden",
    )}
  ></div>
</div>

{@render trigger({
  ref: buttonEl,
  onfocus: handleShow,
  onblur: handleHide,
  onmouseenter: handleShow,
  onmouseleave: handleHide,
})}

<style>
  .tooltip:popover-open {
    opacity: 1;
    scale: 1;
  }

  .tooltip {
    opacity: 0;
    scale: 0.95;

    transition:
      opacity 0.15s,
      scale 0.15s,
      overlay 0.15s allow-discrete,
      display 0.15s allow-discrete;
  }

  @starting-style {
    .tooltip:popover-open {
      opacity: 0;
      scale: 0.95;
    }
  }
</style>
