<script lang="ts">
  import { cn } from "$lib/utils.js";
  import type { HTMLAnchorAttributes } from "svelte/elements";
  import { baseClasses, sizeClasses, variantClasses } from ".";

  type $$Props = HTMLAnchorAttributes & {
    variant?: keyof typeof variantClasses;
    size?: keyof typeof sizeClasses;
  };

  let classes: $$Props["class"] = undefined;
  export let href: $$Props["href"] = undefined;
  export let variant: keyof typeof variantClasses = "link";
  export let size: keyof typeof sizeClasses = "default";
  export { classes as class };
</script>

<a
  class={cn(baseClasses, variantClasses[variant], sizeClasses[size], classes)}
  {href}
  {...$$restProps}
>
  <slot />
</a>
