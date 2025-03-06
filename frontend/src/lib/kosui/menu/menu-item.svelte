<script module lang="ts">
  import { type Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import type { Menu } from ".";
  import { baseClasses, type Variants } from "../base";
  import { mergeProps } from "../merge-props";
  import { noop, type ClassName, type ElementRef } from "../utils";

  export type MenuItemProps = {
    menu: Menu;
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
    menu,
    onSelect,
    children,
    closeOnSelect = true,
    el = $bindable(),
    ref = noop,
    class: className,
    variant = "plain",
    color = "secondary",
    shape = "rounded",
    ...restProps
  }: MenuItemProps = $props();

  function handleSelect() {
    if (closeOnSelect) {
      menu.close();
    }
    onSelect?.();
  }

  $effect(() => {
    if (el) {
      menu.register(el);
      return () => menu.unregister(el);
    }
  });
</script>

<button
  bind:this={el}
  use:ref
  role="menuitem"
  class={twMerge(
    baseClasses({ variant, color, shape, focus: true }),
    "block w-full px-2 py-1 focus:ring-0",
    className,
  )}
  {...mergeProps(restProps, {
    onclick: handleSelect,
    onmouseenter: () => menu.focus(el),
    onfocus: () => menu.focus(el),
  })}
>
  {@render children()}
</button>
