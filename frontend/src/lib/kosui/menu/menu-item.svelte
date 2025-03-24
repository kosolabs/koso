<script module lang="ts">
  import { type Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import { mergeProps } from "../merge-props";
  import { Shortcut } from "../shortcut";
  import { type ClassName, type ElementRef } from "../utils";
  import { getMenuContext } from "./menu-context.svelte";

  export type MenuItemProps = {
    onSelect?: (el: HTMLElement) => void;
    children: Snippet<[]>;
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
    class: className,
    variant = "plain",
    color = "secondary",
    shape = "rounded",
    ...restProps
  }: MenuItemProps = $props();

  const ctx = getMenuContext();

  function handleClick() {
    if (el) {
      onSelect?.(el);
    }
    ctx.close();
  }

  function handleMouseEnter(event: MouseEvent) {
    if (event.target instanceof HTMLElement) {
      ctx.focus(event.target);
    }
  }

  function handleFocus(event: FocusEvent) {
    if (event.target instanceof HTMLElement) {
      ctx.focus(event.target);
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (Shortcut.ENTER.matches(event) || Shortcut.SPACE.matches(event)) {
      event.stopImmediatePropagation();
    }
  }

  $effect(() => {
    if (el) {
      return ctx.add(el);
    }
  });
</script>

<button
  bind:this={el}
  role="menuitem"
  class={twMerge(
    baseClasses({ variant, color, shape, focus: true }),
    "flex w-full items-center gap-1 px-2 py-1 text-left text-sm focus:ring-0",
    className,
  )}
  {...mergeProps(
    {
      onclick: handleClick,
      onmouseenter: handleMouseEnter,
      onfocus: handleFocus,
      onkeydown: handleKeyDown,
    },
    restProps,
  )}
>
  {@render children()}
</button>
