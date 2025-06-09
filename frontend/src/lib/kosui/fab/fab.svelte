<script lang="ts">
  import { twMerge } from "tailwind-merge";
  import { Button, type ButtonProps } from "../button";
  import { type ElementRef } from "../utils";

  export type FabProps = {} & ElementRef & ButtonProps;

  let {
    el = $bindable(),
    class: className,
    icon,
    size = 20,
    variant = "elevated",
    shape = "circle",
    ...restProps
  }: FabProps = $props();
</script>

<div class="grow"></div>
<Button
  bind:el
  class={twMerge("sticky bottom-0 ml-auto p-4 backdrop-blur-sm", className)}
  {variant}
  {shape}
  {icon}
  {size}
  {...restProps}
/>
