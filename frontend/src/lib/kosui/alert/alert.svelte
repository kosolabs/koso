<script lang="ts" module>
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import { noop, type ClassName, type ElementRef } from "../utils";

  export type AlertProps = {
    children?: Snippet;
  } & ElementRef &
    ClassName &
    Variants &
    HTMLAttributes<HTMLDivElement>;
</script>

<script lang="ts">
  let {
    children,
    ref = $bindable(),
    useRef = noop,
    class: className,
    variant = "elevated",
    color = "primary",
    shape = "rounded",
    ...restProps
  }: AlertProps = $props();
</script>

<div
  bind:this={ref}
  use:useRef
  class={twMerge(
    "bg-m3-surface-container",
    baseClasses({ variant, color, shape }),
    "p-3",
    className,
  )}
  {...restProps}
>
  {@render children?.()}
</div>
