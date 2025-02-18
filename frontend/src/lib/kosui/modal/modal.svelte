<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLDialogAttributes } from "svelte/elements";
  import { tv, type ClassValue } from "tailwind-variants";

  export type ToggleEventWithTarget = ToggleEvent & {
    currentTarget: EventTarget & HTMLDialogElement;
  };

  const modalVariants = tv({
    base: "bg-m3-surface-container-high modal-animation m-auto max-w-[min(calc(100%-1em),36em)] min-w-[18em] overflow-hidden rounded-[28px] p-6 shadow-lg",
  });

  type ModalProps = {
    ref?: HTMLDialogElement;
    class?: ClassValue;
    children: Snippet;
  } & HTMLDialogAttributes;
</script>

<script lang="ts">
  let {
    ref = $bindable(),
    open = $bindable(),
    class: className,
    children,
    ontoggle,
    ...props
  }: ModalProps = $props();

  function handleToggle(event: ToggleEventWithTarget) {
    ontoggle?.(event);
    if (event.newState === "closed") {
      open = false;
    } else {
      open = true;
    }
  }

  $effect(() => {
    if (open) {
      console.log("showModal");
      ref?.showModal();
    } else {
      console.log("closeModal");
      ref?.close();
    }
  });
</script>

<dialog
  bind:this={ref}
  ontoggle={handleToggle}
  class={modalVariants({ className })}
  {...props}
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
      animation: close-backdrop 0.15s forwards;
    }
    &[open]::backdrop {
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
