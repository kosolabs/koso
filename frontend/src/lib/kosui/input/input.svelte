<script lang="ts" module>
  import type { HTMLInputAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import type { ClassName, ElementRef } from "../utils";

  export type InputProps = ElementRef &
    ClassName &
    Variants &
    HTMLInputAttributes;
</script>

<script lang="ts">
  let {
    value = $bindable(),
    ref = $bindable(),
    class: className,
    variant = "outlined",
    color = "secondary",
    shape = "rounded",
    ...restProps
  }: InputProps = $props();
</script>

<input
  bind:this={ref}
  bind:value
  class={twMerge(
    "bg-m3-surface-container h-9 px-3",
    baseClasses({ variant, color, shape }),
    className,
  )}
  {...restProps}
/>
