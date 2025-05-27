<script module lang="ts">
  import { type Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { noop, type ClassName, type ElementRef } from "../utils";
  import { setToggleContext, ToggleContext } from "./toggle-context.svelte";

  export type ToggleGroupProps<T> = {
    value?: T;
    onChange?: (value?: T) => void;
    children: Snippet<[]>;
  } & ElementRef &
    ClassName &
    Omit<HTMLAttributes<HTMLDivElement>, "children">;
</script>

<script lang="ts" generics="T">
  let {
    value = $bindable(),
    onChange,
    children,
    el = $bindable(),
    ref = noop,
    class: className,
    ...restProps
  }: ToggleGroupProps<T> = $props();

  setToggleContext(
    new ToggleContext(
      () => value,
      (newValue) => {
        value = newValue;
        onChange?.(newValue);
      },
    ),
  );
</script>

<div bind:this={el} use:ref class={twMerge("flex", className)} {...restProps}>
  {@render children()}
</div>
