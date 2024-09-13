<script lang="ts">
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";
  import { Button as ButtonPrimitive } from "bits-ui";
  import { tv } from "tailwind-variants";
  import { cn } from "./utils";

  export let title: string;
  export let icon;

  const buttonVariants = tv(
    {
      base: "focus-visible:ring-ring inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50 h-8 rounded-md px-3 text-xs",
      variants: {
        variant: {
          default:
            "bg-primary text-primary-foreground hover:bg-primary/90 shadow",
          ghost: "hover:bg-accent hover:text-accent-foreground",
        },
      },
    },
    {
      responsiveVariants: ["sm"],
    },
  );
</script>

<Tooltip.Root>
  <Tooltip.Trigger asChild let:builder>
    <ButtonPrimitive.Root
      builders={[builder]}
      class={cn(
        buttonVariants({ variant: { initial: "ghost", sm: "default" } }),
      )}
      type="button"
      {...$$restProps}
      on:click
      on:keydown
    >
      <svelte:component this={icon} class="w-4 sm:me-2" />
      <div class="text-xs max-sm:hidden">{title}</div>
    </ButtonPrimitive.Root>
  </Tooltip.Trigger>
  <Tooltip.Content>
    {title}
  </Tooltip.Content>
</Tooltip.Root>
