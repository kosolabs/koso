<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { ToggleButton, type ToggleGroup } from ".";
  import { noop, type ClassName, type ElementRef } from "../utils";

  export type ToggleGroupProps = {
    value?: string;
    onChange?: (value?: string) => void;
    children: Snippet<[ToggleGroup]>;
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

  let items: Record<string, ToggleButton> = {};

  export function register(value: string, button: ToggleButton) {
    items[value] = button;
  }

  export function unregister(value: string) {
    delete items[value];
  }

  export function setValue(newValue: string) {
    value = newValue;
  }

  const self: ToggleGroup = { register, unregister, setValue };

  $effect(() => {
    onChange?.(value);
    for (const key in items) {
      items[key].blur();
    }
    if (value && items[value]) {
      items[value].focus();
    }
  });
</script>

<div bind:this={el} use:ref class={twMerge("flex", className)} {...restProps}>
  {@render children(self)}
</div>
