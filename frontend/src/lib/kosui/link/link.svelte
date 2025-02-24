<script module lang="ts">
  import type { Snippet } from "svelte";
  import type {
    HTMLAnchorAttributes,
    HTMLButtonAttributes,
  } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import type { ClassName, ElementRef } from "../utils";

  export type LinkUnderline = "always" | "hover" | "none";
  export type LinkColor =
    | "primary"
    | "secondary"
    | "tertiary"
    | "error"
    | "inherit";

  export type LinkVariants = {
    underline?: LinkUnderline;
    color?: LinkColor;
  };

  export type LinkProps = {
    children: Snippet;
  } & ElementRef &
    ClassName &
    LinkVariants &
    HTMLButtonAttributes &
    HTMLAnchorAttributes;
</script>

<script lang="ts">
  let {
    children,
    ref = $bindable(),
    class: className,
    underline = "hover",
    color = "primary",
    href,
    ...props
  }: LinkProps = $props();

  const classes = twMerge(
    "inline-flex cursor-pointer items-center justify-center gap-1 rounded-md underline-offset-4 hover:opacity-80 focus-visible:ring-1 focus-visible:outline-hidden",
    underline === "always" && "underline",
    underline === "hover" && "hover:underline",
    underline === "none" && "",

    color === "primary" && "text-m3-primary focus-visible:ring-m3-primary",
    color === "secondary" &&
      "text-m3-secondary focus-visible:ring-m3-secondary",
    color === "tertiary" && "text-m3-tertiary focus-visible:ring-m3-tertiary",
    color === "error" && "text-m3-error focus-visible:ring-m3-error",
    color === "inherit" && "focus-visible:ring-m3-primary",
    className,
  );
</script>

{#if href}
  <a bind:this={ref} class={classes} {href} {...props}>
    {@render children()}
  </a>
{:else}
  <button bind:this={ref} class={classes} {...props}>
    {@render children?.()}
  </button>
{/if}
