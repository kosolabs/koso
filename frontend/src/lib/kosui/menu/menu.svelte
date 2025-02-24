<script module lang="ts">
  import { twMerge } from "tailwind-merge";
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
    "bg-m3-surface-container shadow-m3-shadow max-h-[calc(50%)] max-w-[min(calc(100%-1em),17.5em)] overflow-y-scroll rounded-sm py-2 not-disabled:shadow-xs",
    className,
  )}
  {placement}
  {...restProps}
>
  {@render children()}
</Popover>
