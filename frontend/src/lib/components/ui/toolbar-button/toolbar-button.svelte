<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";
  import type { Action } from "$lib/shortcuts";

  const { icon: Icon, title, shortcut, callback }: Action = $props();
</script>

<Tooltip.Root>
  <Tooltip.Trigger asChild let:builder>
    <Button builders={[builder]} variant="ghost" size="sm" onclick={callback}>
      <Icon class="w-4 sm:me-2" />
      <div class="text-xs max-sm:hidden">{title}</div>
    </Button>
  </Tooltip.Trigger>
  <Tooltip.Content>
    <Tooltip.Arrow class="z-50" />
    <div class="flex items-center gap-2">
      <div class="text-secondary">{title}</div>
      {#if shortcut}
        <div class="text-primary-foreground">{shortcut.toString()}</div>
      {/if}
    </div>
  </Tooltip.Content>
</Tooltip.Root>
