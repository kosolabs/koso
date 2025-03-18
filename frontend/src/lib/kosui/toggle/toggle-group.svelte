<script module lang="ts">
  import { type Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { noop, type ClassName, type ElementRef } from "../utils";
  import { newToggleContext } from "./toggle-state.svelte";

  export type ToggleGroupProps = {
    value?: string;
    onChange?: (value?: string) => void;
    children: Snippet<[]>;
  } & ElementRef &
    ClassName &
    Omit<HTMLAttributes<HTMLDivElement>, "children">;
</script>

<script lang="ts">
  let {
    value = $bindable(),
    onChange,
    children,
    el = $bindable(),
    ref = noop,
    class: className,
    ...restProps
  }: ToggleGroupProps = $props();

  newToggleContext(
    () => value,
    (newValue) => {
      value = newValue;
      onChange?.(newValue);
    },
  );
</script>

<div bind:this={el} use:ref class={twMerge("flex", className)} {...restProps}>
  {@render children()}
</div>
