<script lang="ts" module>
  import { X } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import { noop, type ClassName, type ElementRef } from "../utils";

  export type ChipProps = {
    children: Snippet;
    title?: string;
    onClick?: (event: MouseEvent | KeyboardEvent) => void;
    onDelete?: (event: MouseEvent | KeyboardEvent) => void;
  } & ElementRef &
    ClassName &
    Variants;
</script>

<script lang="ts">
  let {
    children,
    title,
    onClick,
    onDelete,
    el = $bindable(),
    ref = noop,
    class: className,
    variant = "filled",
    color = "primary",
    shape = "rounded",
    ...restProps
  }: ChipProps = $props();
</script>

<div
  bind:this={el}
  use:ref
  role="option"
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "inline-flex items-center gap-1 px-1 py-0 text-xs",
    className,
  )}
  {...restProps}
>
  <button onclick={onClick} {title}>
    {@render children()}
  </button>

  {#if onDelete}
    <button
      aria-label="Delete chip"
      class={twMerge(
        "transition-all hover:ring-1 active:scale-95",
        shape === "rounded" && "rounded",
        shape === "circle" && "rounded-full",
      )}
      onclick={onDelete}
    >
      <X size={12} />
    </button>
  {/if}
</div>
