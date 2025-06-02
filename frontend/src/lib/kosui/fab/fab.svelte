<script lang="ts">
  import { twMerge } from "tailwind-merge";
  import { Button, type ButtonProps } from "../button";
  import { type ElementRef } from "../utils";

  export type FabProps = {} & ElementRef & ButtonProps;

  let {
    el = $bindable(),
    class: className,
    icon,
    size = 28,
    variant = "elevated",
    shape = "circle",
    ...restProps
  }: FabProps = $props();
</script>

<div class="grow"></div>
<Button
  bind:el
  class={twMerge("sticky bottom-0 ml-auto backdrop-blur-sm", className)}
  {variant}
  {shape}
  {icon}
  {size}
  {...restProps}
/>
