<script lang="ts" module>
  import { twMerge } from "tailwind-merge";
  import { Box, type BoxProps } from "../box";

  export type AlertProps = BoxProps;
</script>

<script lang="ts">
  let {
    el = $bindable(),
    class: className,
    variant = "elevated",
    color = "primary",
    shape = "rounded",
    backdrop = true,
    ...restProps
  }: AlertProps = $props();
</script>

<Box
  bind:el
  {variant}
  {color}
  {shape}
  {backdrop}
  class={twMerge("p-3", className)}
  {...restProps}
/>
