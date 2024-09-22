<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLAttributes } from "svelte/elements";

  type $$Props = HTMLAttributes<HTMLDivElement>;

  let classes: $$Props["class"] = undefined;
  export { classes as class };
</script>

<div class={cn("flex h-10 w-10", classes)} {...$$restProps}>
  <slot />
</div>
