<script lang="ts">
  import { command, type ActionID } from "$lib/components/ui/command-palette";
  import { NavbarButton } from ".";

  type Props = {
    name: ActionID;
  };

  const { name }: Props = $props();

  let action = $derived(command.get(name));
</script>

{#if action}
  <NavbarButton
    icon={action.icon}
    label={action.title}
    shortcut={action.shortcut?.toString()}
    onclick={action.callback}
  />
{/if}
