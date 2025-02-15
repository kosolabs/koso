<script lang="ts" module>
  import type { Icon } from "lucide-svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { type VariantProps, tv } from "tailwind-variants";
  import { cn } from "../utils";

  export const buttonVariants = tv({
    base: "focus-visible:ring-m3-secondary disabled:text-m3-on-surface/38 flex h-10 items-center gap-2 rounded-[20px] px-6 text-sm text-nowrap transition-all focus-visible:outline-hidden",
    variants: {
      variant: {
        elevated:
          "bg-m3-surface-container-low shadow-m3-shadow text-m3-primary hover:bg-m3-surface-container-low-hover focus-visible:bg-m3-surface-container-low-focus active:bg-m3-surface-container-low-active disabled:bg-m3-on-surface/12 not-disabled:shadow-xs not-disabled:active:scale-95",
        filled:
          "bg-m3-primary text-m3-on-primary shadow-m3-shadow hover:bg-m3-primary-hover focus-visible:bg-m3-primary-focus active:bg-m3-primary-active disabled:bg-m3-on-surface/12 not-disabled:not-active:hover:shadow-xs not-disabled:active:scale-95",
        tonal:
          "bg-m3-secondary-container shadow-m3-shadow text-m3-on-secondary-container hover:bg-m3-secondary-container-hover focus-visible:bg-m3-secondary-container-focus active:bg-m3-secondary-container-active disabled:bg-m3-on-surface/12 not-disabled:not-active:hover:shadow-xs not-disabled:active:scale-95",
        outline:
          "text-m3-primary not-disabled:hover:bg-m3-primary/8 focus-visible:bg-m3-primary/10 not-disabled:active:bg-m3-primary/12 focus-visible:outline-m3-primary outline-m3-outline disabled:outline-m3-on-surface/12 outline not-disabled:active:scale-95",
        ghost:
          "text-m3-primary not-disabled:hover:bg-m3-primary/8 focus-visible:bg-m3-primary/10 not-disabled:active:bg-m3-primary/12 disabled:outline-m3-on-surface/12 not-disabled:active:scale-95",
      },
    },
    defaultVariants: {
      variant: "outline",
    },
  });

  export type ButtonVariants = VariantProps<typeof buttonVariants>;

  export type ButtonProps = HTMLButtonAttributes &
    ButtonVariants & {
      ref?: HTMLElement;
      icon?: typeof Icon;
    };
</script>

<script lang="ts">
  let {
    class: className,
    variant,
    ref = $bindable(),
    icon: IconComponent,
    children,
    ...restProps
  }: ButtonProps = $props();
</script>

<button
  bind:this={ref}
  class={cn(buttonVariants({ variant }), className)}
  {...restProps}
>
  {#if IconComponent}
    <IconComponent size={18} class="ml-[-4px]" />
  {/if}
  {@render children?.()}
</button>
