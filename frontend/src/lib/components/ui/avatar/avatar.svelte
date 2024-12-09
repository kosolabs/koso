<script lang="ts">
  import { cn } from "$lib/utils.js";
  import { Avatar as AvatarPrimitive } from "bits-ui";

  let {
    class: className,
    ref = $bindable(null),
    loadingStatus = $bindable("loading"),
    ...restProps
  }: AvatarPrimitive.RootProps = $props();
</script>

<AvatarPrimitive.Root
  bind:loadingStatus
  bind:ref
  class={cn(
    "relative flex size-10 shrink-0 overflow-hidden rounded-full",
    className,
  )}
  {...restProps}
/>
