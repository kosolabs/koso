<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import type { ClassName } from "../utils";

  export type HeaderProps = {
    children: Snippet;
  } & ClassName &
    Variants &
    HTMLAttributes<HTMLDivElement>;
</script>

<script lang="ts">
  let {
    children,
    class: className,
    variant = "plain",
    color = "primary",
    shape = "rounded",
    ...restProps
  }: HeaderProps = $props();
</script>

<div
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "px-2 py-1 text-xs uppercase",
    className,
  )}
  {...restProps}
>
  {@render children()}
</div>
