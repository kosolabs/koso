<script module lang="ts">
  import type { Snippet } from "svelte";
  import type {
    HTMLAnchorAttributes,
    HTMLButtonAttributes,
  } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import { noop, type ClassName, type ElementRef } from "../utils";

  export type LinkProps = {
    children: Snippet;
  } & ElementRef &
    ClassName &
    Variants &
    HTMLButtonAttributes &
    HTMLAnchorAttributes;
</script>

<script lang="ts">
  let {
    children,
    el = $bindable(),
    ref = noop,
    class: className,
    variant = "text",
    color = "primary",
    shape = "rounded",
    underline = "hover",
    href,
    ...props
  }: LinkProps = $props();

  const classes = twMerge(
    baseClasses({ variant, color, shape, underline, hover: true, focus: true }),
    variant !== "text" &&
      "flex h-9 items-center justify-center gap-2 text-nowrap transition-all active:scale-95",
    "cursor-pointer",
    className,
  );
</script>

{#if href}
  <a bind:this={el} use:ref class={classes} {href} {...props}>
    {@render children()}
  </a>
{:else}
  <button bind:this={el} use:ref class={classes} {...props}>
    {@render children()}
  </button>
{/if}
