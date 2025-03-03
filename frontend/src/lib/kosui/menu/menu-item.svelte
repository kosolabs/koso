<script module lang="ts">
  import { type Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import type { Menu } from ".";
  import { baseClasses, type Variants } from "../base";
  import { mergeProps } from "../merge-props";
  import { noop, type ClassName, type ElementRef } from "../utils";

  export type MenuItemProps = {
    menuRef: Menu;
    onSelect?: () => void;
    closeOnSelect?: boolean;
    children: Snippet<[]>;
  } & ElementRef &
    ClassName &
    Variants &
    Omit<HTMLButtonAttributes, "type" | "value">;
</script>

<script lang="ts">
  let {
    menuRef,
    onSelect,
    children,
    closeOnSelect = true,
    ref = $bindable(),
    useRef = noop,
    class: className,
    variant = "plain",
    color = "secondary",
    shape = "rounded",
    ...restProps
  }: MenuItemProps = $props();

  function handleSelect() {
    if (closeOnSelect) {
      menuRef.close();
    }
    onSelect?.();
  }

  $effect(() => {
    if (ref) {
      menuRef.register(ref);
      return () => menuRef.unregister(ref);
    }
  });
</script>

<button
  bind:this={ref}
  use:useRef
  class={twMerge(
    baseClasses({ variant, color, shape, focus: true }),
    "block w-full px-2 py-1 focus:ring-0",
    className,
  )}
  {...mergeProps(restProps, {
    onclick: handleSelect,
    onmouseenter: () => menuRef.focus(ref),
    onfocus: () => menuRef.focus(ref),
  })}
>
  {@render children()}
</button>
