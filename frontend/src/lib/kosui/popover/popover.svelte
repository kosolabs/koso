<script lang="ts" module>
  import * as floatingUi from "@floating-ui/dom";
  import { type Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { scale } from "svelte/transition";
  import { twMerge } from "tailwind-merge";
  import { events } from "..";
  import { mergeProps } from "../merge-props";
  import { Shortcut } from "../shortcut";
  import {
    noop,
    type ClassName,
    type ElementRef,
    type ToggleEventWithTarget,
  } from "../utils";

  export type PopoverProps = {
    arrow?: boolean;
    placement?: floatingUi.Placement;
    strategy?: floatingUi.Strategy;
    open?: boolean;
    anchorEl?: HTMLElement;
    onKeydownWhileOpen?: (event: KeyboardEvent) => void;
    children: Snippet;
  } & ClassName &
    ElementRef &
    HTMLAttributes<HTMLDivElement>;
</script>

<script lang="ts">
  let {
    arrow = false,
    placement = "top",
    strategy = "fixed",
    open = $bindable(false),
    anchorEl,
    children,
    onKeydownWhileOpen,
    el: popoverEl = $bindable(),
    ref = noop,
    class: className,
    ...restProps
  }: PopoverProps = $props();

  let arrowEl: HTMLDivElement | undefined = $state();

  function handleEscape(event: KeyboardEvent) {
    if (popoverEl && Shortcut.ESCAPE.matches(event)) {
      popoverEl.hidePopover();
      anchorEl?.focus();
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  }

  function handleClickOutside(event: MouseEvent) {
    if (
      popoverEl &&
      event.target instanceof Node &&
      !popoverEl.contains(event.target) &&
      (!anchorEl || !anchorEl.contains(event.target))
    ) {
      popoverEl.hidePopover();
    }
  }

  function handleClickInside(event: MouseEvent) {
    event.stopImmediatePropagation();
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
      events.on("keydown", onKeydownWhileOpen);
      events.on("mousedown", handleClickOutside);

      return () => {
        events.remove("keydown", handleEscape);
        events.remove("keydown", onKeydownWhileOpen);
        events.remove("mousedown", handleClickOutside);
      };
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

  if (!children) {
    console.log(children);
  }
</script>

{#if open}
  <div
    bind:this={popoverEl}
    use:ref
    popover="manual"
    class={twMerge(className)}
    transition:scale={{ duration: 150, start: 0.95 }}
    {...mergeProps(restProps, { ontoggle, onclick: handleClickInside })}
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
