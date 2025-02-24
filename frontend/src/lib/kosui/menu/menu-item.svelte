<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
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
  let type: HTMLButtonAttributes["type"] = $derived(
    value === undefined ? "button" : "submit",
  );
</script>

<button
  class={twMerge(
    "focus-visible:ring-m3-secondary text-m3-on-surface disabled:text-m3-on-surface/38 hover:bg-m3-surface-hover focus-visible:bg-m3-surface-focus active:bg-m3-surface-active focus-visible:outline-m3-secondary block h-12 w-full min-w-28 px-3 text-start",
    className,
  )}
  {type}
  {value}
  {...restProps}
>
  {@render children()}
</button>
