<script module lang="ts">
  import { type Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import { mergeProps } from "../merge-props";
  import { type ClassName, type ElementRef } from "../utils";
  import { getCommandContext } from "./command-context.svelte";

  export type CommandItemProps = {
    onSelect?: (el: HTMLElement) => void;
    children: Snippet<[]>;
  } & ElementRef &
    ClassName &
    Variants &
    HTMLAttributes<HTMLDivElement>;
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
  }: CommandItemProps = $props();

  const ctx = getCommandContext();

  function handleClick() {
    if (el) {
      onSelect?.(el);
    }
  }

  function handleMouseEnter() {
    ctx.focused = el;
  }

  $effect(() => {
    if (el) {
      return ctx.add(el);
    }
  });
</script>

<div
  bind:this={el}
  role="option"
  tabindex="-1"
  aria-selected={ctx.focused === el}
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "aria-selected:bg-m3-secondary/15 flex w-full cursor-pointer items-center gap-1 px-2 py-1 text-left text-sm focus:ring-0 disabled:bg-transparent",
    className,
  )}
  {...mergeProps(restProps, {
    onclick: handleClick,
    onmouseenter: handleMouseEnter,
  })}
>
  {@render children()}
</div>
