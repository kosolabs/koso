<script lang="ts" module>
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import type { ClassName, ElementRef } from "../utils";

  export const alertVariants = [
    "elevated",
    "filled",
    "tonal",
    "outlined",
    "plain",
  ] as const;

  export const alertColors = [
    "primary",
    "secondary",
    "tertiary",
    "error",
  ] as const;

  export type AlertVariant = (typeof alertVariants)[number];
  export type AlertColor = (typeof alertColors)[number];

  export type AlertVariants = {
    variant?: AlertVariant;
    color?: AlertColor;
  };

  export type AlertProps = {
    children?: Snippet;
  } & ElementRef &
    ClassName &
    AlertVariants &
    HTMLAttributes<HTMLDivElement>;
</script>

<script lang="ts">
  let {
    children,
    ref = $bindable(),
    class: className,
    variant = "elevated",
    color = "primary",
    ...restProps
  }: AlertProps = $props();
</script>

<div
  bind:this={ref}
  class={twMerge(
    "shadow-m3-shadow/20 rounded-m3 bg-m3-surface-container p-3",
    variant === "elevated" && "bg-m3-surface-container-low shadow",
    variant === "filled" && "",
    variant === "tonal" && "",
    variant === "outlined" && "ring-1",
    variant === "plain" && "",

    color === "primary" && "text-m3-primary",
    color === "secondary" && "text-m3-secondary",
    color === "tertiary" && "text-m3-tertiary",
    color === "error" && "text-m3-error",

    variant === "filled" &&
      color === "primary" &&
      "bg-m3-primary text-m3-on-primary",
    variant === "filled" &&
      color === "secondary" &&
      "bg-m3-secondary text-m3-on-secondary",
    variant === "filled" &&
      color === "tertiary" &&
      "bg-m3-tertiary text-m3-on-tertiary",
    variant === "filled" && color === "error" && "bg-m3-error text-m3-on-error",

    variant === "tonal" &&
      color === "primary" &&
      "bg-m3-primary-container text-m3-on-primary-container",
    variant === "tonal" &&
      color === "secondary" &&
      "bg-m3-secondary-container text-m3-on-secondary-container",
    variant === "tonal" &&
      color === "tertiary" &&
      "bg-m3-tertiary-container text-m3-on-tertiary-container",
    variant === "tonal" &&
      color === "error" &&
      "bg-m3-error-container text-m3-on-error-container",

    className,
  )}
  {...restProps}
>
  {@render children?.()}
</div>
