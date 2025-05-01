<script lang="ts">
  import { twMerge } from "tailwind-merge";
  import { Button, type ButtonProps } from "../button";
  import { type ClassName, type ElementRef } from "../utils";

  export type FabProps = { reserve?: boolean } & ElementRef &
    ClassName &
    ButtonProps;

  let {
    reserve: space = false,
    el = $bindable(),
    class: className,
    icon,
    size = 28,
    variant = "elevated",
    shape = "circle",
    ...restProps
  }: FabProps = $props();
</script>

{#if space && el}
  <div style:height={`${el.getBoundingClientRect().height}px`}></div>
{/if}

<div class="absolute right-2 bottom-2">
  <Button
    bind:el
    class={twMerge("fixed -translate-full", className)}
    {variant}
    {shape}
    {icon}
    {size}
    {...restProps}
  />
</div>
