<script lang="ts">
  import { command, type ActionID } from "$lib/components/ui/command-palette";
  import { twMerge } from "tailwind-merge";
  import { NavbarButton } from ".";

  type Props = {
    name: ActionID;
    desktop?: boolean;
  };

  const { name, desktop = false }: Props = $props();

  let action = $derived(command.get(name));
</script>

{#if action && action.enabled()}
  <NavbarButton
    class={twMerge(desktop && "max-sm:hidden")}
    icon={action.icon}
    label={action.title}
    shortcut={action.shortcut?.toString()}
    onclick={action.callback}
  />
{/if}
