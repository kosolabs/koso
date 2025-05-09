<script lang="ts">
  import { command, type ActionID } from "$lib/components/ui/command-palette";
  import { Button } from "$lib/kosui/button";
  import { twMerge } from "tailwind-merge";

  type Props = {
    name: ActionID;
    desktop?: boolean;
  };

  const { name, desktop = false }: Props = $props();

  let action = $derived(command.get(name));
</script>

{#if action && action.enabled()}
  <Button
    variant="plain"
    shape="circle"
    icon={action.icon}
    size={20}
    aria-label={action.title}
    onclick={action.callback}
    class={twMerge(desktop && "max-sm:hidden")}
  >
    {#snippet tooltip()}
      <div class="flex items-center gap-2">
        {action.title}
        {#if action.shortcut}
          <div class="font-bold">
            {action.shortcut.toString()}
          </div>
        {/if}
      </div>
    {/snippet}
  </Button>
{/if}
