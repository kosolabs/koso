<script module lang="ts">
  import { twMerge } from "tailwind-merge";
  import { baseClasses } from "../base";
  import { Popover, type PopoverProps } from "../popover";
  import type { ClassName } from "../utils";

  export type MenuProps = {} & ClassName & PopoverProps;
</script>

<script lang="ts">
  let {
    class: className,
    open = $bindable(),
    placement = "bottom",
    children,
    ...restProps
  }: MenuProps = $props();
</script>

<Popover
  bind:open
  class={twMerge(
    baseClasses({ variant: "plain", color: "primary", shape: "rounded" }),
    "bg-m3-surface-container-lowest border p-1 text-sm shadow",
    className,
  )}
  {placement}
  {...restProps}
>
  {@render children()}
</Popover>
