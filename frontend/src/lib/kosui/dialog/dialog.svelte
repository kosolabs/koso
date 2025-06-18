<script module lang="ts">
  import type { Icon } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import { mergeComponentProps } from "../merge-props";
  import { Modal, type ModalProps } from "../modal";

  export type DialogProps<T> = {
    ref?: HTMLDialogElement;
    onSelect?: (value: T | undefined) => void;
    icon?: typeof Icon;
    title?: string;
    children: Snippet;
    actions?: Snippet<[{ onSelect: (value: T | undefined) => void }]>;
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

  function handleSelect(value: T | undefined) {
    onSelect?.(value);
    open = false;
  }

  function handleCancel() {
    onSelect?.(undefined);
  }
</script>

<Modal
  bind:ref
  bind:open
  class={twMerge("w-[min(calc(100%-1em),36em)]", className)}
  {...mergeComponentProps(
    Modal,
    { role: "dialog", "aria-modal": "true", onCancel: handleCancel },
    restProps,
  )}
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
