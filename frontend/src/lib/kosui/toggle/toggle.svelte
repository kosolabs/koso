<script lang="ts" module>
  import type { ButtonProps } from "../button";

  export type ToggleProps = {
    pressed: boolean;
  } & Omit<ButtonProps, "variant">;
</script>

<script lang="ts">
  import { Button } from "../button";

  let { pressed, el = $bindable(), ...restProps }: ToggleProps = $props();
</script>

<Button bind:el variant={pressed ? "filled" : "outlined"} {...restProps} />
