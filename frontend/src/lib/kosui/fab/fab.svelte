<script lang="ts">
  import { twMerge } from "tailwind-merge";
  import { Button, type ButtonProps } from "../button";
  import type { ClassName } from "../utils";

  type FabProps = ClassName & ButtonProps;

  let {
    class: className,
    icon,
    size = 28,
    variant = "elevated",
    shape = "circle",
    ...restProps
  }: FabProps = $props();
</script>

<div class="absolute right-2 bottom-2">
  <Button
    class={twMerge("fixed -translate-full", className)}
    {variant}
    {shape}
    {icon}
    {size}
    {...restProps}
  />
</div>
