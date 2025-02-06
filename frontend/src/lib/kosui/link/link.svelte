<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLAnchorAttributes } from "svelte/elements";
  import { tv, type ClassValue, type VariantProps } from "tailwind-variants";

  const linkVariants = tv({
    base: "focus-visible:ring-ring inline-flex cursor-pointer items-center justify-center gap-1 rounded-md underline-offset-4 focus-visible:ring-1 focus-visible:outline-hidden",
    variants: {
      underline: {
        always: "underline",
        hover: "hover:underline",
        never: "",
      },
      color: {
        primary: "text-primary",
        secondary: "text-secondary",
        inherit: "",
      },
    },
  });

  export type LinkUnderline = VariantProps<typeof linkVariants>["underline"];
  export type LinkColor = VariantProps<typeof linkVariants>["color"];

  export type LinkProps = HTMLAnchorAttributes & {
    children: Snippet;
    class?: ClassValue;
    underline?: LinkUnderline;
    color?: LinkColor;
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
