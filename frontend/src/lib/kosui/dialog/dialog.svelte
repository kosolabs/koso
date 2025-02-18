<script module lang="ts">
  import type { Icon } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import { tv, type ClassValue } from "tailwind-variants";
  import { Modal, type ModalProps } from "../modal";
  import type { ToggleEventWithTarget } from "../utils";

  const dialogVariants = tv({});

  export type DialogProps = {
    ref?: HTMLDialogElement;
    onSelect?: (result: string) => void;
    class?: ClassValue;
    icon?: typeof Icon;
    title?: string;
    children: Snippet;
    actions?: Snippet;
  } & ModalProps;
</script>

<script lang="ts">
  let {
    ref = $bindable(),
    open = $bindable(),
    onSelect = () => {},
    ontoggle,
    class: className,
    icon: IconComponent,
    title,
    children,
    actions,
    ...props
  }: DialogProps = $props();

  function handleToggle(event: ToggleEventWithTarget<HTMLDialogElement>) {
    ontoggle?.(event);
    if (!ref) throw new Error("ref should be defined!");
    if (event.newState === "closed") {
      onSelect(ref.returnValue);
    } else {
      ref.returnValue = "";
    }
  }
</script>

<Modal
  bind:ref
  bind:open
  ontoggle={(event) => handleToggle(event)}
  class={dialogVariants({ className })}
  {...props}
>
  <form method="dialog">
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
          {@render actions()}
        </div>
      {/if}
    </div>
  </form>
</Modal>
