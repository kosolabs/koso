<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLDialogAttributes } from "svelte/elements";
  import { tv, type VariantProps } from "tailwind-variants";
  import { events } from "..";
  import { baseVariants } from "../base";
  import { mergeProps } from "../merge-props";
  import { startTooltipCooldown } from "../tooltip";
  import { type ClassName, type ToggleEventWithTarget } from "../utils";

  export const modalVariants = tv({
    extend: baseVariants,
    base: "bg-m3-surface-container-high modal-animation m-auto max-w-[min(calc(100%-1em),36em)] min-w-[18em] overflow-hidden rounded-lg p-5 shadow-lg",
  });

  export type ModalVariants = VariantProps<typeof modalVariants>;

  export type ModalProps = {
    ref?: HTMLDialogElement;
    enableEscapeHandler?: boolean;
    children: Snippet;
  } & ClassName &
    ModalVariants &
    HTMLDialogAttributes;
</script>

<script lang="ts">
  let {
    ref = $bindable(),
    open = $bindable(),
    enableEscapeHandler = false,
    class: className,
    variant,
    color,
    scale,
    children,
    ...restProps
  }: ModalProps = $props();

  function handleEscape(event: KeyboardEvent) {
    if (event.key === "Escape") {
      open = false;
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  }

  function ontoggle(event: ToggleEventWithTarget<HTMLDialogElement>) {
    startTooltipCooldown();
    if (event.newState === "closed") {
      open = false;
    } else {
      open = true;
    }
  }

  $effect(() => {
    if (open) {
      ref?.showModal();
      if (enableEscapeHandler) {
        events.on("keydown", handleEscape);
      }
    } else {
      ref?.close();
      if (enableEscapeHandler) {
        events.remove("keydown", handleEscape);
      }
    }
  });

  const mergedProps = $derived(mergeProps({ ontoggle }, restProps));
</script>

<dialog
  bind:this={ref}
  class={modalVariants({ variant, color, scale, className })}
  {...mergedProps}
>
  {@render children()}
</dialog>

<style>
  .modal-animation {
    transition:
      overlay 0.15s allow-discrete,
      display 0.15s allow-discrete;

    animation: close-dialog 0.15s forwards;
    &[open] {
      animation: open-dialog 0.15s forwards;
    }

    &::backdrop {
      z-index: 50;
      animation: close-backdrop 0.15s forwards;
    }
    &[open]::backdrop {
      z-index: 50;
      animation: open-backdrop 0.15s forwards;
    }
  }

  @keyframes open-backdrop {
    from {
      background: rgba(0, 0, 0, 0);
      backdrop-filter: blur(0px);
    }
    to {
      background: rgba(0, 0, 0, 0.5);
      backdrop-filter: blur(2px);
    }
  }

  @keyframes close-backdrop {
    from {
      background: rgba(0, 0, 0, 0.5);
      backdrop-filter: blur(2px);
    }
    to {
      background: rgba(0, 0, 0, 0);
      backdrop-filter: blur(0px);
    }
  }

  @keyframes open-dialog {
    from {
      opacity: 0;
      scale: 0.95;
    }
    to {
      opacity: 1;
      scale: 1;
    }
  }

  @keyframes close-dialog {
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
