<script lang="ts">
  import { cn } from "$lib/utils";
  import { Tooltip as TooltipPrimitive } from "bits-ui";

  const { class: classes, ...props }: TooltipPrimitive.ArrowProps = $props();
</script>

<div class="bg-primary">
  <TooltipPrimitive.Arrow class={cn(classes)} {...props} />
</div>
