<script lang="ts">
  import { type Action, MenuItem, ShortcutBadge } from "kosui";

  type Props = {
    action: Action;
  };
  const { action }: Props = $props();
</script>

{#if action && action.enabled()}
  {@const Icon = action.icon}
  <MenuItem class="gap-2" onSelect={action.callback} title={action.description}>
    <Icon size={16} />
    {action.name}
    {#if action.shortcut}
      <ShortcutBadge class="ml-auto" shortcut={action.shortcut} />
    {/if}
  </MenuItem>
{/if}
