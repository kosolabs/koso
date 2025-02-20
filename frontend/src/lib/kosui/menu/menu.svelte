<script module lang="ts">
  import { tv, type VariantProps } from "tailwind-variants";
  import { Popover, type PopoverProps } from "../popover";

  export const menuVariants = tv({
    base: "bg-m3-surface-container shadow-m3-shadow max-h-[calc(50%)] max-w-[min(calc(100%-1em),17.5em)] overflow-y-scroll rounded-sm py-2 transition-all not-disabled:shadow-xs",
  });

  export type MenuVariants = VariantProps<typeof menuVariants>;

  export type MenuProps = {} & MenuVariants & PopoverProps;
</script>

<script lang="ts">
  let {
    open = $bindable(),
    placement = "bottom",
    class: className,
    children,
    ...restProps
  }: MenuProps = $props();
</script>

<Popover
  bind:open
  class={menuVariants({ className })}
  {placement}
  {...restProps}
>
  {@render children()}
</Popover>
