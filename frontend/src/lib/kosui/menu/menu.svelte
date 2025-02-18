<script module lang="ts">
  import { tv, type VariantProps } from "tailwind-variants";
  import { Popover, type PopoverProps } from "../popover";

  export const menuVariants = tv({
    base: "bg-m3-surface-container shadow-m3-shadow rounded-sm py-2 transition-all not-disabled:shadow-xs max-w-[calc(100%-1em),17.5em)] overflow-y-scroll max-h-[calc(50%)]",
  });

  export type MenuVariants = VariantProps<typeof menuVariants>;

  export type MenuProps = {
  } & PopoverProps;
</script>

<script lang="ts">
  let {
    open = $bindable(),
    placement = "bottom",
    children,
    ...restProps
  }: MenuProps = $props();
</script>

<Popover bind:open class={menuVariants({})} {placement} {...restProps}>
  {@render children()}
</Popover>
