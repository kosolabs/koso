<script lang="ts">
  import { cn } from "$lib/utils";
  import { Tooltip as TooltipPrimitive } from "bits-ui0";

  const { class: classes, ...props }: TooltipPrimitive.ArrowProps = $props();
</script>

<TooltipPrimitive.Arrow class={cn("bg-primary", classes)} {...props} />
