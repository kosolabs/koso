<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses } from "../base";
  import type { ClassName } from "../utils";

  export type MenuButtonProps = {
    children: Snippet<[]>;
  } & ClassName &
    Omit<HTMLButtonAttributes, "type">;
</script>

<script lang="ts">
  let {
    class: className,
    children,
    value,
    ...restProps
  }: MenuButtonProps = $props();
</script>

<button
  class={twMerge(
    baseClasses({ variant: "plain", color: "primary", shape: "rounded" }),
    "block",
    className,
  )}
  {value}
  {...restProps}
>
  {@render children()}
</button>
