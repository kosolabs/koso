<script lang="ts">
  import { twMerge } from "tailwind-merge";
  import { Box, type BoxProps } from "../box";

  export type BadgeProps = {
    verticalAlignment?: "top" | "bottom";
    horizontalAlignment?: "left" | "right";
    content?: string;
  } & BoxProps;

  let {
    verticalAlignment = "top",
    horizontalAlignment = "right",
    content,
    children,
    class: className,
    variant = "filled",
    color = "primary",
    centered = true,
  }: BadgeProps = $props();
</script>

<div class="relative">
  {#if content}
    <Box
      {variant}
      {color}
      {centered}
      shape="circle"
      class={twMerge(
        "absolute z-10 size-5 p-1 text-xs",

        verticalAlignment === "top" && "top-0 -translate-y-1/2",
        verticalAlignment === "bottom" && "bottom-0 translate-y-1/2",
        horizontalAlignment === "left" && "left-0 -translate-x-1/2",
        horizontalAlignment === "right" && "right-0 translate-x-1/2",

        className,
      )}>{content}</Box
    >
  {/if}
  {@render children?.()}
</div>
