<script lang="ts">
  import * as Command from "$lib/components/ui/command/index.js";
  import { Strokes } from "$lib/components/ui/stroke";
  import type { Action } from ".";

  type Props = {
    open: boolean;
    actions: Action[];
  };
  let { open = $bindable(), actions }: Props = $props();

  let filter: string = $state("");

  const filteredActions = $derived(
    actions.filter((action) =>
      action.title.toLocaleLowerCase().includes(filter),
    ),
  );
</script>

<Command.Dialog bind:open shouldFilter={false}>
  <Command.Input
    bind:value={filter}
    placeholder="Type a command or search..."
  />
  <Command.List>
    <Command.Empty>No results found.</Command.Empty>
    {#each filteredActions as action}
      {@const { title, icon: Icon, callback, shortcut } = action}
      <Command.Item value={title} onSelect={callback}>
        <Icon class="mr-2 h-4 w-4" />
        {title}
        {#if shortcut}
          <Strokes class="ml-auto" binding={shortcut} />
        {/if}
      </Command.Item>
    {/each}
  </Command.List>
</Command.Dialog>
