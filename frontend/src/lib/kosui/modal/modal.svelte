<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLDialogAttributes } from "svelte/elements";
  import { scale } from "svelte/transition";
  import { twMerge } from "tailwind-merge";
  import { events } from "..";
  import { mergeProps } from "../merge-props";
  import { Shortcut } from "../shortcut";
  import { type ClassName } from "../utils";

  export type ModalProps = {
    ref?: HTMLDialogElement;
    onCancel?: () => void;
    children: Snippet;
  } & ClassName &
    HTMLDialogAttributes;
</script>

<script lang="ts">
  let {
    ref = $bindable(),
    open = $bindable(),
    onCancel,
    children,
    class: className,
    ...restProps
  }: ModalProps = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (Shortcut.ESCAPE.matches(event)) {
      open = false;
      onCancel?.();
      event.preventDefault();
    }
    event.stopImmediatePropagation();
  }

  function handleClickOutside(event: MouseEvent) {
    if (ref && ref === event.target) {
      const rect = ref.getBoundingClientRect();
      if (
        event.clientY < rect.top ||
        event.clientY > rect.top + rect.height ||
        event.clientX < rect.left ||
        event.clientX > rect.left + rect.width
      ) {
        open = false;
        onCancel?.();
        event.preventDefault();
        event.stopImmediatePropagation();
      }
    }
  }

  $effect(() => {
    if (ref) {
      ref.showModal();
      events.on("keydown", handleKeydown);
      events.on("mousedown", handleClickOutside);

      return () => {
        events.remove("keydown", handleKeydown);
        events.remove("mousedown", handleClickOutside);
      };
    }
  });
</script>

{#if open}
  <dialog
    bind:this={ref}
    class={twMerge("m-auto rounded-lg p-5 shadow-lg", className)}
    transition:scale={{ duration: 150, start: 0.95 }}
    {...mergeProps(restProps, {
      onintrostart: () => {
        if (ref) {
          ref.classList.remove("backdrop-out");
          ref.classList.add("backdrop-in");
        }
      },
      onoutrostart: () => {
        if (ref) {
          ref.classList.remove("backdrop-in");
          ref.classList.add("backdrop-out");
        }
      },
    })}
  >
    {@render children()}
  </dialog>
{/if}

<style>
  .backdrop-in::backdrop {
    animation: backdrop-in 150ms forwards;
  }

  .backdrop-out::backdrop {
    animation: backdrop-out 150ms forwards;
  }

  @keyframes backdrop-in {
    from {
      background: rgba(0, 0, 0, 0);
      backdrop-filter: blur(0px);
    }
    to {
      background: rgba(0, 0, 0, 0.5);
      backdrop-filter: blur(2px);
    }
  }

  @keyframes backdrop-out {
    from {
      background: rgba(0, 0, 0, 0.5);
      backdrop-filter: blur(2px);
    }
    to {
      background: rgba(0, 0, 0, 0);
      backdrop-filter: blur(0px);
    }
  }
</style>
