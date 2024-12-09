<script lang="ts">
  import * as Command from "$lib/components/ui/command";
  import { ShortcutChips } from "$lib/components/ui/shortcut";
  import { Shortcut, type Action } from "$lib/shortcuts";
  import { match } from "$lib/utils";

  type Props = {
    open: boolean;
    actions: Action[];
  };
  let { open = $bindable(), actions }: Props = $props();

  let query: string = $state("");
  const filteredActions = $derived(
    actions
      .filter((action) => action.enabled())
      .filter(
        (action) =>
          match(action.title, query) || match(action.description, query),
      ),
  );
</script>

<Command.Dialog
  bind:open
  shouldFilter={false}
  portalProps={{ to: document.body }}
  onkeydown={(event) => {
    event.stopPropagation();
    if (Shortcut.CANCEL.matches(event)) {
      query = "";
      open = false;
    }
  }}
>
  <Command.Input bind:value={query} placeholder="Type a command or search..." />
  <Command.List>
    <Command.Empty>No results found.</Command.Empty>
    {#each filteredActions as action}
      {@const { title, description, icon: Icon, callback, shortcut } = action}
      <Command.Item
        value={title}
        onSelect={() => {
          callback();
          query = "";
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
