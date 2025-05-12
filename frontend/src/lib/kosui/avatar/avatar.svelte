<script lang="ts" module>
  import { twMerge } from "tailwind-merge";
  import { Box, type BoxProps } from "../box";

  export type AvatarProps = {
    src?: string;
    alt?: string;
  } & BoxProps;
</script>

<script lang="ts">
  let {
    src,
    alt,
    children,
    el = $bindable(),
    class: className,
    variant = "tonal",
    color = "secondary",
    shape = "rounded",
    centered = true,
    ...restProps
  }: AvatarProps = $props();

  let error: boolean = $state(false);
</script>

<Box
  bind:el
  {variant}
  {color}
  {shape}
  {centered}
  class={twMerge("aspect-square h-9 overflow-clip text-xl", className)}
  {...restProps}
>
  {#if src && !error}
    <img
      {src}
      {alt}
      onerror={() => (error = true)}
      referrerpolicy="no-referrer"
    />
  {:else}
    {@render children?.()}
  {/if}
</Box>
