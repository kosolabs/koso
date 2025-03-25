<script lang="ts">
  import type { ButtonProps } from "$lib/kosui/button";
  import { Button } from "$lib/kosui/button";

  type NavbarButtonProps = ButtonProps;

  let {
    icon,
    size = 20,
    variant = "plain",
    shape = "circle",
    ...restProps
  }: NavbarButtonProps = $props();
</script>

<Button {variant} {shape} class="p-2" {icon} {size} {...restProps} />
