<script lang="ts" module>
  import * as floatingUi from "@floating-ui/dom";
  import { type Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { tv, type ClassValue, type VariantProps } from "tailwind-variants";
  import { events } from "..";
  import type { ToggleEventWithTarget } from "../utils";

  export const popoverVariants = tv({
    base: "popover-animation",
  });

  export type PopoverVariants = VariantProps<typeof popoverVariants>;

  export type PopoverProps = {
    class?: ClassValue;
    arrow?: boolean;
    placement?: floatingUi.Placement;
    strategy?: floatingUi.Strategy;
    open?: boolean;
    anchorEl?: HTMLElement;
    enableEscapeHandler?: boolean;
    children: Snippet;
  } & PopoverVariants &
    HTMLAttributes<HTMLDivElement>;
</script>

<script lang="ts">
  let {
    class: className,
    arrow = false,
    placement = "top",
    strategy = "fixed",
    open = $bindable(false),
    enableEscapeHandler = false,
    anchorEl,
    children,
    ontoggle,
    ...restProps
  }: PopoverProps = $props();

  let popoverEl: HTMLDivElement | undefined = $state();
  let arrowEl: HTMLDivElement | undefined = $state();

  function handleEscape(event: KeyboardEvent) {
    if (event.key === "Escape") {
      open = false;
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  }

  function handleToggle(event: ToggleEventWithTarget<HTMLDivElement>) {
    ontoggle?.(event);
    if (event.newState === "closed") {
      open = false;
    } else {
      open = true;
    }
  }

  $effect(() => {
    if (open) {
      popoverEl?.showPopover();
      if (enableEscapeHandler) {
        events.on("keydown", handleEscape);
      }
    } else {
      popoverEl?.hidePopover();
      if (enableEscapeHandler) {
        events.remove("keydown", handleEscape);
      }
    }
  });

  async function updatePosition() {
    if (anchorEl && popoverEl && arrowEl) {
      const computedPosition = await floatingUi.computePosition(
        anchorEl,
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

  $effect(() => {
    if (anchorEl && popoverEl) {
      return floatingUi.autoUpdate(anchorEl, popoverEl, updatePosition);
    }
  });
</script>

<div
  bind:this={popoverEl}
  popover="auto"
  role="tooltip"
  class={popoverVariants({ className })}
  ontoggle={handleToggle}
  {...restProps}
>
  {@render children()}
  <div
    bind:this={arrowEl}
    class={twMerge(
      "bg-m3-inverse-surface absolute -z-10 size-2 rotate-45",
      arrow ? "block" : "hidden",
    )}
  ></div>
</div>

<style>
  .popover-animation {
    transition:
      overlay 0.15s allow-discrete,
      display 0.15s allow-discrete;

    animation: close-tooltip 0.15s forwards;
  }

  .popover-animation:popover-open {
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
