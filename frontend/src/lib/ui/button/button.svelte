<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { baseClasses, sizeClasses, variantClasses } from ".";

  type Props = HTMLButtonAttributes & {
    variant?: keyof typeof variantClasses;
    size?: keyof typeof sizeClasses;
  };

  let {
    children,
    variant = "default",
    size = "default",
    class: classes,
    ...props
  }: Props = $props();
</script>

<button
  class={cn(baseClasses, variantClasses[variant], sizeClasses[size], classes)}
  {...props}
>
  {#if children}
    {@render children()}
  {/if}
</button>
