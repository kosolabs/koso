<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLImgAttributes } from "svelte/elements";

  type $$Props = HTMLImgAttributes;

  let classes: $$Props["class"] = undefined;
  export let src: $$Props["src"] = undefined;
  export let alt: $$Props["alt"] = undefined;
  export { classes as class };
</script>

<img
  {src}
  {alt}
  class={cn("aspect-square h-full w-full", classes)}
  {...$$restProps}
/>
