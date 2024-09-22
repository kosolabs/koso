<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLAttributes } from "svelte/elements";

  type $$Props = HTMLAttributes<HTMLSpanElement>;

  let classes: $$Props["class"] = undefined;
  export { classes as class };
</script>

<span
  class={cn("flex h-full w-full items-center justify-center bg-muted", classes)}
  {...$$restProps}
>
  <slot />
</span>
