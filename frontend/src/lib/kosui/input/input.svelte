<script lang="ts" module>
  import type { HTMLInputAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import { noop, type ClassName, type ElementRef } from "../utils";

  export type InputProps = ElementRef &
    ClassName &
    Variants &
    HTMLInputAttributes;
</script>

<script lang="ts">
  let {
    value = $bindable(),
    el = $bindable(),
    ref = noop,
    class: className,
    variant = "outlined",
    color = "secondary",
    shape = "rounded",
    ...restProps
  }: InputProps = $props();
</script>

<input
  bind:this={el}
  use:ref
  bind:value
  class={twMerge(
    "bg-m3-surface-container",
    baseClasses({ variant, color, shape, hover: true, focus: true }),

    (variant === "outlined" || variant === "plain") &&
      "hover:bg-m3-surface-container-low focus-visible:bg-m3-surface-container-low",

    "h-9 px-3",
    className,
  )}
  {...restProps}
/>
