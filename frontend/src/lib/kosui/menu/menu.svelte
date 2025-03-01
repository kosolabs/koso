<script module lang="ts">
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import type { Menu } from ".";
  import { baseClasses, type Variants } from "../base";
  import { Popover, type PopoverProps } from "../popover";
  import type { ClassName } from "../utils";

  export type MenuProps = {
    content: Snippet<[Menu]>;
  } & ClassName &
    Variants &
    Omit<PopoverProps, "children">;
</script>

<script lang="ts">
  let {
    content,
    class: className,
    variant = "elevated",
    color = "primary",
    shape = "rounded",
    open = $bindable(),
    placement = "bottom",
    ...restProps
  }: MenuProps = $props();

  export function close() {
    open = false;
  }

  const self: Menu = { close };
</script>

<Popover
  bind:open
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "bg-m3-surface-container-lowest border p-1 shadow",
    className,
  )}
  {placement}
  {...restProps}>{@render content(self)}</Popover
>
