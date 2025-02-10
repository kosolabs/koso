<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLAnchorAttributes } from "svelte/elements";
  import { tv, type ClassValue, type VariantProps } from "tailwind-variants";

  export const linkVariants = tv({
    base: "inline-flex cursor-pointer items-center justify-center gap-1 rounded-md underline-offset-4 focus-visible:ring-1 focus-visible:outline-hidden",
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
        inherit: "focus-visible:ring-m3-primary",
      },
    },
    defaultVariants: {
      underline: "always",
      color: "primary",
    },
  });

  export type LinkVariants = VariantProps<typeof linkVariants>;

  export type LinkProps = HTMLAnchorAttributes &
    LinkVariants & {
      children: Snippet;
      class?: ClassValue;
    };
</script>

<script lang="ts">
  const {
    children,
    class: className,
    underline = "hover",
    color = "primary",
    ...props
  }: LinkProps = $props();
</script>

<a class={linkVariants({ underline, color, className })} {...props}>
  {@render children()}
</a>
