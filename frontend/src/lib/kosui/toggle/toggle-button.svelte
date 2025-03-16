<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { type ToggleButton } from ".";
  import { type Variants } from "../base";
  import { mergeProps } from "../merge-props";
  import { noop, type ClassName, type ElementRef } from "../utils";
  import ToggleGroup from "./toggle-group.svelte";

  export type ToggleButtonProps = {
    value: string;
    toggleGroup: ToggleGroup;
    children: Snippet;
  } & ElementRef &
    ClassName &
    Omit<Variants, "variant"> &
    HTMLButtonAttributes;
</script>

<script lang="ts">
  let {
    value,
    toggleGroup,
    children,
    el = $bindable(),
    ref = noop,
    class: className,
    color = "primary",
    shape = "rounded",
    ...restProps
  }: ToggleButtonProps = $props();

  let focused: boolean = $state(false);

  export function focus() {
    focused = true;
  }

  export function blur() {
    focused = false;
  }

  const self: ToggleButton = { blur, focus };

  $effect(() => {
    toggleGroup.register(value, self);
    return () => toggleGroup.unregister(value);
  });
</script>

<button
  bind:this={el}
  use:ref
  role="option"
  aria-selected={focused}
  class={twMerge(
    "bg-md-background text-md-on-surface flex items-center gap-1 border-y border-l px-4 py-1.5 text-sm last:border-r",

    shape === "square" && "",
    shape === "rounded" && "first:rounded-l-md last:rounded-r-md",
    shape === "circle" && "first:rounded-l-full last:rounded-r-full",

    color === "primary" &&
      "aria-selected:bg-m3-primary aria-selected:text-m3-on-primary border-m3-primary text-m3-primary",
    color === "secondary" &&
      "aria-selected:bg-m3-secondary aria-selected:text-m3-on-secondary border-m3-secondary text-m3-secondary",
    color === "tertiary" &&
      "aria-selected:bg-m3-tertiary aria-selected:text-m3-on-tertiary border-m3-tertiary text-m3-tertiary",
    color === "error" &&
      "aria-selected:bg-m3-error aria-selected:text-m3-on-error border-m3-error text-m3-error",

    className,
  )}
  {...mergeProps(restProps, { onclick: () => toggleGroup.setValue(value) })}
>
  {@render children()}
</button>
