<script lang="ts" module>
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import type { ClassName, ElementRef } from "../utils";

  export type AvatarProps = {
    src?: string;
    alt?: string;
    children?: Snippet;
  } & ElementRef &
    ClassName &
    Variants &
    HTMLAttributes<HTMLDivElement>;
</script>

<script lang="ts">
  let {
    src,
    alt,
    children,
    ref = $bindable(),
    class: className,
    variant = "tonal",
    color = "secondary",
    shape = "rounded",
    ...restProps
  }: AvatarProps = $props();
</script>

<div
  bind:this={ref}
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "flex aspect-square size-9 items-center justify-center overflow-clip text-xl",
    className,
  )}
  {...restProps}
>
  {#if src}
    <img {src} {alt} />
  {:else}
    {@render children?.()}
  {/if}
</div>
