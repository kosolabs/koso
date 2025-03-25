<script lang="ts">
  import { command } from "$lib/kosui/command";
  import { NavbarButton } from ".";

  type Props = {
    name: string;
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
