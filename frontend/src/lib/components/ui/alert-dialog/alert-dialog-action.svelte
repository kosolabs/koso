<script lang="ts">
  import { cn } from "$lib/utils.js";
  import { AlertDialog as AlertDialogPrimitive } from "bits-ui0";
  import { buttonVariants } from ".";

  type $$Props = AlertDialogPrimitive.ActionProps;
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  type $$Events = AlertDialogPrimitive.ActionEvents;

  let className: $$Props["class"] = undefined;
  export { className as class };
</script>

<AlertDialogPrimitive.Action
  class={cn(buttonVariants(), className)}
  {...$$restProps}
  on:click
  on:keydown
  let:builder
>
  <slot {builder} />
</AlertDialogPrimitive.Action>
