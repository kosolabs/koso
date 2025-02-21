<script lang="ts" module>
  import * as floatingUi from "@floating-ui/dom";
  import { type Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import type { TransitionConfig } from "svelte/transition";
  import { twMerge } from "tailwind-merge";
  import { tv, type ClassValue, type VariantProps } from "tailwind-variants";
  import { events } from "..";
  import { mergeProps } from "../merge-props";
  import { type ToggleEventWithTarget } from "../utils";

  export const popoverVariants = tv({});

  export type PopoverVariants = VariantProps<typeof popoverVariants>;

  export type PopoverProps = {
    class?: ClassValue;
    arrow?: boolean;
    placement?: floatingUi.Placement;
    strategy?: floatingUi.Strategy;
    open?: boolean;
    anchorEl?: HTMLElement;
    children: Snippet;
  } & PopoverVariants &
    HTMLAttributes<HTMLDivElement>;

  function kosui(node: Element, config?: TransitionConfig): TransitionConfig {
    const { duration = 150, ...restProps } = config ?? {};
    return {
      duration,
      ...restProps,
      css: (t: number) => {
        return `opacity: ${t};scale: ${0.95 + 0.05 * t};`;
      },
    };
  }
</script>

<script lang="ts">
  let {
    class: className,
    arrow = false,
    placement = "top",
    strategy = "fixed",
    open = $bindable(false),
    anchorEl,
    children,
    ...restProps
  }: PopoverProps = $props();

  let popoverEl: HTMLDivElement | undefined = $state();
  let arrowEl: HTMLDivElement | undefined = $state();

  function handleEscape(event: KeyboardEvent) {
    if (event.key === "Escape") {
      popoverEl?.hidePopover();
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  }

  function ontoggle(event: ToggleEventWithTarget<HTMLDivElement>) {
    if (event.newState === "closed") {
      open = false;
    } else {
      open = true;
    }
  }

  $effect(() => {
    if (popoverEl) {
      popoverEl.showPopover();
      events.on("keydown", handleEscape);
    } else {
      events.remove("keydown", handleEscape);
    }
  });

  $effect(() => {
    if (anchorEl && popoverEl) {
      return floatingUi.autoUpdate(anchorEl, popoverEl, updatePosition);
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

  const mergedProps = $derived(mergeProps({ ontoggle }, restProps));
</script>

{#if open}
  <div
    bind:this={popoverEl}
    popover="manual"
    role="tooltip"
    class={popoverVariants({ className })}
    transition:kosui
    {...mergedProps}
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
{/if}
