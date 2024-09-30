<script lang="ts">
  import * as Command from "$lib/components/ui/command/index.js";
  import { Strokes } from "$lib/components/ui/stroke";
  import { KeyBinding } from "$lib/key-binding";

  type Action = {
    title: string;
    icon: any;
    callback: (value: string) => void;
    shortcut?: KeyBinding;
  };

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

<Command.Dialog bind:open shouldFilter={false} portal={null}>
  <Command.Input
    bind:value={filter}
    placeholder="Type a command or search..."
  />
  <Command.List>
    <Command.Empty>No results found.</Command.Empty>
    {#each filteredActions as action}
      {@const { title, icon: Icon, callback, shortcut } = action}
      <Command.Item
        value={title}
        onSelect={(value) => {
          open = false;
          callback(value);
        }}
      >
        <Icon class="mr-2 h-4 w-4" />
        {title}
        {#if shortcut}
          <Strokes class="ml-auto" binding={shortcut} />
        {/if}
      </Command.Item>
    {/each}
  </Command.List>
</Command.Dialog>
