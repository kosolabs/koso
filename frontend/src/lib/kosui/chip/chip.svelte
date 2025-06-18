<script lang="ts" module>
  import { X } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";

  export type ChipProps = {
    title?: string;
    onClick?: (event: MouseEvent | KeyboardEvent) => void;
    onDelete?: (event: MouseEvent | KeyboardEvent) => void;
    children: Snippet;
  } & BoxProps;
</script>

<script lang="ts">
  import { Box, type BoxProps } from "../box";

  let {
    title,
    onClick,
    onDelete,
    children,
    el = $bindable(),
    class: className,
    variant = "filled",
    color = "primary",
    shape = "rounded",
    ...restProps
  }: ChipProps = $props();
</script>

<Box
  bind:el
  role="option"
  {variant}
  {color}
  {shape}
  class={twMerge("inline-flex gap-1 px-1 py-0 text-xs", className)}
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
</Box>
