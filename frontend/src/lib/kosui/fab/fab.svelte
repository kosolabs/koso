<script lang="ts">
  import { twMerge, type ClassNameValue } from "tailwind-merge";
  import { Button, type ButtonProps } from "../button";
  import { type ElementRef } from "../utils";

  export type FabProps = {
    verticalAlignment?: "top" | "bottom";
    horizontalAlignment?: "left" | "right";
    reserve?: boolean;
    reserveClass?: ClassNameValue;
    positionClass?: ClassNameValue;
  } & ElementRef &
    ButtonProps;

  let {
    verticalAlignment = "bottom",
    horizontalAlignment = "right",
    reserve: space = false,
    reserveClass,
    positionClass,
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
  <div
    class={twMerge(reserveClass)}
    style:height={`${el.getBoundingClientRect().height}px`}
  ></div>
{/if}

<div
  class={twMerge(
    "absolute right-0 bottom-0 m-2",
    verticalAlignment === "top" && "top-0",
    verticalAlignment === "bottom" && "bottom-0",
    horizontalAlignment === "left" && "left-0",
    horizontalAlignment === "right" && "right-0",
    positionClass,
  )}
>
  <Button
    bind:el
    class={twMerge(
      "fixed backdrop-blur-sm",
      verticalAlignment === "bottom" && "-translate-y-full",
      horizontalAlignment === "right" && "-translate-x-full",
      className,
    )}
    {variant}
    {shape}
    {icon}
    {size}
    {...restProps}
  />
</div>
