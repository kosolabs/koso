<script lang="ts">
  import { getRegistryContext } from "$lib/components/ui/command-palette";
  import { Button } from "kosui";
  import { twMerge } from "tailwind-merge";

  type Props = {
    name: string;
    desktop?: boolean;
  };

  const { name, desktop = false }: Props = $props();

  const command = getRegistryContext();
  let action = $derived(command.get(name));
</script>

{#if action && action.enabled()}
  <Button
    variant="plain"
    shape="circle"
    icon={action.icon}
    size={20}
    aria-label={action.description}
    onclick={action.callback}
    class={twMerge(desktop && "max-sm:hidden")}
  >
    {#snippet tooltip()}
      <div class="flex items-center gap-2">
        {action.description}
        {#if action.shortcut}
          <div class="font-bold">
            {action.shortcut.toString()}
          </div>
        {/if}
      </div>
    {/snippet}
  </Button>
{/if}
