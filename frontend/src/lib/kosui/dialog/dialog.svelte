<script module lang="ts">
  import type { Icon } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import { events } from "..";
  import { mergeProps } from "../merge-props";
  import { Modal, type ModalProps } from "../modal";

  type DialogProps<T> = {
    ref?: HTMLDialogElement;
    onSelect?: (value?: T) => void;
    icon?: typeof Icon;
    title?: string;
    children: Snippet;
    actions?: Snippet<[{ onSelect: (value: T) => void }]>;
  } & ModalProps;
</script>

<script lang="ts" generics="T">
  let {
    ref = $bindable(),
    open = $bindable(),
    onSelect,
    class: className,
    icon: IconComponent,
    title,
    children,
    actions,
    ...restProps
  }: DialogProps<T> = $props();

  function handleSelect(value: T) {
    onSelect?.(value);
    open = false;
  }

  function handleEscape(event: KeyboardEvent) {
    if (event.key === "Escape") {
      open = false;
      onSelect?.();
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  }

  $effect(() => {
    if (open) {
      events.on("keydown", handleEscape);
    } else {
      events.remove("keydown", handleEscape);
    }
  });

  const mergedProps = $derived(mergeProps({ ontoggle }, restProps));
</script>

<Modal
  bind:ref
  bind:open
  class={twMerge("w-[min(calc(100%-1em),36em)]", className)}
  {...mergedProps}
>
  <div class={twMerge("flex flex-col gap-4")}>
    {#if IconComponent}
      <IconComponent class="mx-auto" />
    {/if}
    {#if title}
      <div class={twMerge("text-xl", IconComponent ? "text-center" : "")}>
        {title}
      </div>
    {/if}
    <div>
      {@render children()}
    </div>
    {#if actions}
      <div class="mt-2 flex place-content-end gap-2">
        {@render actions({ onSelect: handleSelect })}
      </div>
    {/if}
  </div>
</Modal>
