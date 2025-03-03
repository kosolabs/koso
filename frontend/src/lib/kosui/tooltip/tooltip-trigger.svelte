<script lang="ts" module>
  import { type Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import type { Variants } from "../base";
  import { noop, type ClassName, type ElementRef } from "../utils";

  export type TooltipTriggerProps = {
    children?: Snippet;
  } & ElementRef &
    ClassName &
    Variants &
    HTMLAttributes<HTMLElement>;
</script>

<script lang="ts">
  let {
    children,
    el = $bindable(),
    ref = noop,
    class: className,
    ...restProps
  }: TooltipTriggerProps = $props();
</script>

<div bind:this={el} use:ref class={twMerge(className)} {...restProps}>
  {@render children?.()}
</div>
