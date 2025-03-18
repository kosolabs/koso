<script lang="ts">
  import {
    Command,
    CommandDivider,
    CommandInput,
    CommandItem,
  } from "$lib/kosui/command";
  import { Modal } from "$lib/kosui/modal";
  import ShortcutBadge from "$lib/kosui/shortcut/shortcut-badge.svelte";
  import { type Action } from "$lib/shortcuts";
  import { match } from "$lib/utils";
  import { SearchIcon } from "lucide-svelte";

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

  function handleSelect(action: Action) {
    action.callback();
    query = "";
    open = false;
  }
</script>

<Modal
  bind:open
  class="bg-m3-surface-container h-[min(40%,24em)] w-[min(calc(100%-1em),36em)] rounded-lg p-0"
>
  <Command class="flex h-full flex-col">
    <div class="flex items-center px-2">
      <SearchIcon size={16} />
      <CommandInput
        bind:value={query}
        placeholder="Type a command or search..."
      />
    </div>
    <CommandDivider />
    <div class="h-full overflow-scroll">
      {#if filteredActions.length > 0}
        {#each filteredActions as action (action.title)}
          <CommandItem
            title={action.title}
            onSelect={() => handleSelect(action)}
          >
            <action.icon class="mr-2 h-4 w-4" />
            {action.description}
            {#if action.shortcut}
              <ShortcutBadge class="ml-auto" shortcut={action.shortcut} />
            {/if}
          </CommandItem>
        {/each}
      {:else}
        <div class="text-center">No results found.</div>
      {/if}
    </div>
  </Command>
</Modal>
