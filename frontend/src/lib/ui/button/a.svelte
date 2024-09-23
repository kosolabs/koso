<script lang="ts">
  import { cn } from "$lib/utils.js";
  import type { HTMLAnchorAttributes } from "svelte/elements";
  import { baseClasses, sizeClasses, variantClasses } from ".";

  type Props = HTMLAnchorAttributes & {
    variant?: keyof typeof variantClasses;
    size?: keyof typeof sizeClasses;
  };

  let {
    children,
    variant = "link",
    size = "default",
    class: classes,
    ...props
  }: Props = $props();
</script>

<a
  class={cn(baseClasses, variantClasses[variant], sizeClasses[size], classes)}
  {...props}
>
  {#if children}
    {@render children()}
  {/if}
</a>
