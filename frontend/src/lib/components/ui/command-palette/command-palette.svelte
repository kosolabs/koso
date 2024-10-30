<script lang="ts">
  import * as Command from "$lib/components/ui/command";
  import { ShortcutChips } from "$lib/components/ui/shortcut";
  import { Shortcut, type Action } from "$lib/shortcuts";

  type Props = {
    open: boolean;
    actions: Action[];
  };
  let { open = $bindable(), actions }: Props = $props();

  let filter: string = $state("");

  const filteredActions = $derived(
    actions
      .filter((action) => action.enabled())
      .filter(
        (action) =>
          action.title
            .toLocaleLowerCase()
            .includes(filter.toLocaleLowerCase()) ||
          action.description
            .toLocaleLowerCase()
            .includes(filter.toLocaleLowerCase()),
      ),
  );
</script>

<Command.Dialog
  bind:open
  shouldFilter={false}
  portal={null}
  onkeydown={(event) => {
    event.stopPropagation();
    if (Shortcut.CANCEL.matches(event)) {
      filter = "";
      open = false;
    }
  }}
>
  <Command.Input
    bind:value={filter}
    placeholder="Type a command or search..."
  />
  <Command.List>
    <Command.Empty>No results found.</Command.Empty>
    {#each filteredActions as action}
      {@const { title, description, icon: Icon, callback, shortcut } = action}
      <Command.Item
        value={title}
        onSelect={() => {
          callback();
          filter = "";
          open = false;
        }}
      >
        <Icon class="mr-2 h-4 w-4" />
        {description}
        {#if shortcut}
          <ShortcutChips class="ml-auto" {shortcut} />
        {/if}
      </Command.Item>
    {/each}
  </Command.List>
</Command.Dialog>
