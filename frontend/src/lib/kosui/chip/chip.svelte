<script lang="ts" module>
  import { X } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import { type Variants } from "../base";
  import type { ClassName, ElementRef } from "../utils";

  export type ChipProps = {
    children: Snippet;
    onClick?: (event: MouseEvent | KeyboardEvent) => void;
    onDelete?: (event: MouseEvent | KeyboardEvent) => void;
  } & ElementRef &
    ClassName &
    Variants &
    ButtonProps;
</script>

<script lang="ts">
  import { Button, type ButtonProps } from "../button";

  let {
    children,
    onClick,
    onDelete,
    ref = $bindable(),
    class: className,
    variant = "filled",
    color = "primary",
    shape = "rounded",
    ...restProps
  }: ChipProps = $props();
</script>

<Button
  bind:ref
  class={twMerge("inline-flex items-center gap-1 px-1 py-0 text-xs", className)}
  onclick={onClick}
  {variant}
  {color}
  {shape}
  {...restProps}
>
  {@render children()}

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
</Button>
