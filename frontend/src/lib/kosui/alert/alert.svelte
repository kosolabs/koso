<script lang="ts" module>
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import type { ClassName, ElementRef } from "../utils";

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
    class: className,
    variant = "elevated",
    color = "primary",
    shape = "rounded",
    ...restProps
  }: AlertProps = $props();
</script>

<div
  bind:this={ref}
  class={twMerge(
    "bg-m3-surface-container p-3",
    baseClasses({ variant, color, shape }),
    className,
  )}
  {...restProps}
>
  {@render children?.()}
</div>
