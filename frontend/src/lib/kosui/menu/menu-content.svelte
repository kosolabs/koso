<script module lang="ts">
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import { mergeComponentProps } from "../merge-props";
  import { Popover, type PopoverProps } from "../popover";
  import { type ClassName } from "../utils";
  import { getMenuContext } from "./menu-context.svelte";

  export type MenuProps = {
    children: Snippet;
  } & ClassName &
    Variants &
    Omit<PopoverProps, "children" | "open">;
</script>

<script lang="ts">
  let {
    children,
    class: className,
    variant = "elevated",
    color = "primary",
    shape = "rounded",
    placement = "bottom",
    ...restProps
  }: MenuProps = $props();

  const ctx = getMenuContext();
</script>

<Popover
  bind:open={ctx.open}
  role="menu"
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "bg-m3-surface-container-highest max-h-[60%] border p-1 shadow",
    className,
  )}
  {placement}
  anchorEl={ctx.anchorEl}
  {...mergeComponentProps(Popover, restProps, {
    onmouseleave: () => ctx.blur(),
    onKeydownWhileOpen: (event) => ctx.handleKeyDown(event),
  })}
>
  {@render children()}
</Popover>
