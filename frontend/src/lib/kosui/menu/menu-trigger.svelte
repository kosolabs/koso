<script lang="ts" module>
  import { type Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import type { Variants } from "../base";
  import { noop, type ClassName, type ElementRef } from "../utils";

  export type MenuTriggerProps = {
    children?: Snippet;
  } & ElementRef &
    ClassName &
    Variants &
    HTMLButtonAttributes;
</script>

<script lang="ts">
  let {
    children,
    el = $bindable(),
    ref = noop,
    class: className,
    ...restProps
  }: MenuTriggerProps = $props();
</script>

<button bind:this={el} use:ref class={twMerge(className)} {...restProps}>
  {@render children?.()}
</button>
