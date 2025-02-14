<script lang="ts">
  import { Button } from "$lib/kosui/button";
  import PlainTooltip from "$lib/kosui/tooltip/plain-tooltip.svelte";
  import type { Action } from "$lib/shortcuts";

  const { icon, title, description, shortcut, callback }: Action = $props();
</script>

<PlainTooltip arrow>
  {#snippet trigger(ref, props)}
    <Button
      bind:ref={ref.value}
      variant="ghost"
      {icon}
      onclick={callback}
      {...props}
    >
      <div class="max-sm:hidden">{title}</div>
    </Button>
  {/snippet}
  <div class="flex items-center gap-2">
    <div class="text-primary-foreground">{description}</div>
    {#if shortcut}
      <div class="text-primary-foreground font-bold">
        {shortcut.toString()}
      </div>
    {/if}
  </div>
</PlainTooltip>
