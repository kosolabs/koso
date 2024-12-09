<script lang="ts">
  import { AlertDialog as AlertDialogPrimitive } from "bits-ui0";

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  type $$Props = AlertDialogPrimitive.PortalProps;
</script>

<AlertDialogPrimitive.Portal {...$$restProps}>
  <slot />
</AlertDialogPrimitive.Portal>
