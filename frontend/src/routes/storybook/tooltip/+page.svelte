<script lang="ts">
  import { Button } from "$lib/kosui/button";
  import { PlainTooltip } from "$lib/kosui/tooltip";
  import { Shortcut } from "$lib/shortcuts";

  const shortcut = new Shortcut({ key: "Enter", shift: true });
</script>

<div class="h-[1024px] w-[1024px]">
  <PlainTooltip arrow>
    {#snippet trigger(buttonProps)}
      <Button {...buttonProps}>Hover for Tooltip</Button>
    {/snippet}
    <div class="flex items-center gap-2">
      <div class="text-primary-foreground">Some Description</div>
      <div class="text-primary-foreground font-bold">
        {shortcut.toString()}
      </div>
    </div>
  </PlainTooltip>
</div>
