<script lang="ts">
  import type { ButtonProps } from "$lib/kosui/button";
  import { Button } from "$lib/kosui/button";

  type NavbarButtonProps = { label?: string; shortcut?: string } & ButtonProps;

  let {
    icon,
    label,
    shortcut,
    size = 20,
    variant = "plain",
    shape = "circle",
    ...restProps
  }: NavbarButtonProps = $props();
</script>

<Button {variant} {shape} {icon} {size} aria-label={label} {...restProps}>
  {#snippet tooltip()}
    <div class="flex items-center gap-2">
      {label}
      {#if shortcut}
        <div class="font-bold">
          {shortcut.toString()}
        </div>
      {/if}
    </div>
  {/snippet}
</Button>
