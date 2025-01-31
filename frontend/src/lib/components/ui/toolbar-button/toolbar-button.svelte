<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import type { Action } from "$lib/shortcuts";

  const {
    icon: Icon,
    title,
    description,
    shortcut,
    callback,
  }: Action = $props();
</script>

<Tooltip.Provider>
  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button {...props} variant="ghost" onclick={callback}>
          <Icon />
          <div class="max-sm:hidden">{title}</div>
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Portal>
      <Tooltip.Content>
        <Tooltip.Arrow class="z-50" />
        <div class="flex items-center gap-2">
          <div class="text-primary-foreground">{description}</div>
          {#if shortcut}
            <div class="text-primary-foreground font-bold">
              {shortcut.toString()}
            </div>
          {/if}
        </div>
      </Tooltip.Content>
    </Tooltip.Portal>
  </Tooltip.Root>
</Tooltip.Provider>
