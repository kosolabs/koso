<script module lang="ts">
  import type { Snippet } from "svelte";
  import type {
    HTMLAnchorAttributes,
    HTMLButtonAttributes,
  } from "svelte/elements";
  import { tv, type ClassValue, type VariantProps } from "tailwind-variants";
  import type { ElementRef } from "../utils";

  export const linkVariants = tv({
    base: "inline-flex cursor-pointer items-center justify-center gap-1 rounded-md underline-offset-4 hover:opacity-80 focus-visible:ring-1 focus-visible:outline-hidden",
    variants: {
      underline: {
        always: "underline",
        hover: "hover:underline",
        none: "",
      },
      color: {
        primary: "text-m3-primary focus-visible:ring-m3-primary",
        secondary: "text-m3-secondary focus-visible:ring-m3-secondary",
        tertiary: "text-m3-tertiary focus-visible:ring-m3-tertiary",
        error: "text-m3-error focus-visible:ring-m3-error",
        inherit: "focus-visible:ring-m3-primary",
      },
    },
    defaultVariants: {
      underline: "always",
      color: "primary",
    },
  });

  export type LinkVariants = VariantProps<typeof linkVariants>;

  export type LinkProps = ElementRef &
    HTMLButtonAttributes &
    HTMLAnchorAttributes &
    LinkVariants & {
      children: Snippet;
      class?: ClassValue;
    };
</script>

<script lang="ts">
  let {
    children,
    class: className,
    ref = $bindable(),
    underline = "hover",
    color = "primary",
    href,
    ...props
  }: LinkProps = $props();
</script>

{#if href}
  <a
    bind:this={ref}
    class={linkVariants({ underline, color, className })}
    {href}
    {...props}
  >
    {@render children()}
  </a>
{:else}
  <button
    bind:this={ref}
    class={linkVariants({ underline, color, className })}
    {...props}
  >
    {@render children?.()}
  </button>
{/if}
