<script lang="ts">
  import { cn } from "$lib/utils.js";
  import { AlertDialog as AlertDialogPrimitive } from "bits-ui";
  import { buttonVariants } from ".";

  type $$Props = AlertDialogPrimitive.CancelProps;
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  type $$Events = AlertDialogPrimitive.CancelEvents;

  let className: $$Props["class"] = undefined;
  export { className as class };
</script>

<AlertDialogPrimitive.Cancel
  class={cn(buttonVariants({ variant: "outline" }), "mt-2 sm:mt-0", className)}
  {...$$restProps}
  on:click
  on:keydown
  let:builder
>
  <slot {builder} />
</AlertDialogPrimitive.Cancel>
