<script lang="ts">
  import { cn } from "$lib/utils.js";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { baseClasses, sizeClasses, variantClasses } from ".";

  type $$Props = HTMLButtonAttributes & {
    variant?: keyof typeof variantClasses;
    size?: keyof typeof sizeClasses;
  };

  let classes: $$Props["class"] = undefined;
  export let variant: keyof typeof variantClasses = "default";
  export let size: keyof typeof sizeClasses = "default";
  export { classes as class };
</script>

<button
  class={cn(baseClasses, variantClasses[variant], sizeClasses[size], classes)}
  {...$$restProps}
  on:click
  on:keydown
>
  <slot />
</button>
