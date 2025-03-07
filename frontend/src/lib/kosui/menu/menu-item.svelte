<script module lang="ts">
  import { type Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import { mergeProps } from "../merge-props";
  import { noop, type ClassName, type ElementRef } from "../utils";

  export type MenuItemProps = {
    onSelect?: (el: HTMLElement) => void;
    children: Snippet<[]>;
    unref?: (el: HTMLElement) => void;
  } & ElementRef &
    ClassName &
    Variants &
    Omit<HTMLButtonAttributes, "type" | "value">;
</script>

<script lang="ts">
  let {
    onSelect,
    children,
    el = $bindable(),
    ref = noop,
    unref = noop,
    class: className,
    variant = "plain",
    color = "secondary",
    shape = "rounded",
    ...restProps
  }: MenuItemProps = $props();

  $effect(() => {
    if (el) {
      ref(el);
      return () => el && unref(el);
    }
  });
</script>

<button
  bind:this={el}
  class={twMerge(
    baseClasses({ variant, color, shape, focus: true }),
    "block w-full px-2 py-1 focus:ring-0",
    className,
  )}
  {...mergeProps(restProps, {
    onclick: () => el && onSelect?.(el),
  })}
>
  {@render children()}
</button>
