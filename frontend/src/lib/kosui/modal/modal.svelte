<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLDialogAttributes } from "svelte/elements";
  import { scale } from "svelte/transition";
  import { twMerge } from "tailwind-merge";
  import { events } from "..";
  import { Shortcut } from "../shortcut";
  import { type ClassName } from "../utils";

  export type ModalProps = {
    ref?: HTMLDialogElement;
    useEscapeKey?: boolean;
    children: Snippet;
  } & ClassName &
    HTMLDialogAttributes;
</script>

<script lang="ts">
  let {
    ref = $bindable(),
    useEscapeKey = false,
    open = $bindable(),
    class: className,
    children,
    ...restProps
  }: ModalProps = $props();

  $effect(() => {
    if (ref) {
      ref.showModal();
    }
  });

  function handleEscape(event: KeyboardEvent) {
    const ESCAPE = new Shortcut({ key: "Escape" });
    if (ESCAPE.matches(event)) {
      open = false;
    }
  }

  $effect(() => {
    if (useEscapeKey) {
      return events.on("keydown", handleEscape);
    }
  });
</script>

{#if open}
  <dialog
    bind:this={ref}
    class={twMerge(
      "bg-m3-surface-container-high m-auto max-w-[min(calc(100%-1em),36em)] min-w-72 overflow-hidden rounded-lg p-5 shadow-lg",
      className,
    )}
    {...restProps}
    transition:scale={{ duration: 150, start: 0.95 }}
    onintrostart={() => {
      if (ref) {
        ref.classList.remove("backdrop-out");
        ref.classList.add("backdrop-in");
      }
    }}
    onoutrostart={() => {
      if (ref) {
        ref.classList.remove("backdrop-in");
        ref.classList.add("backdrop-out");
      }
    }}
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
