<script lang="ts">
  import { type ActionID } from "$lib/components/ui/command-palette";
  import type { Action } from "$lib/kosui/command";
  import { MenuItem } from "$lib/kosui/menu";
  import { ShortcutBadge } from "$lib/kosui/shortcut";

  type Props = {
    action: Action<ActionID>;
  };
  const { action }: Props = $props();
</script>

{#if action && action.enabled()}
  {@const Icon = action.icon}
  <MenuItem class="gap-2" onSelect={action.callback} title={action.description}>
    <Icon size={16} />
    {action.title}
    {#if action.shortcut}
      <ShortcutBadge class="ml-auto" shortcut={action.shortcut} />
    {/if}
  </MenuItem>
{/if}
