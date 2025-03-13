<script lang="ts">
  import { Command, CommandInput, CommandItem } from "$lib/kosui/command";
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
</script>

<Modal bind:open class="h-96 min-h-96 rounded p-0 sm:w-96 sm:min-w-96">
  <Command class="flex h-full flex-col">
    {#snippet input(command)}
      <div class="flex items-center px-2">
        <SearchIcon size={16} />
        <CommandInput
          bind:value={query}
          {command}
          placeholder="Type a command or search..."
        />
      </div>
    {/snippet}
    {#snippet content(command)}
      <div class="h-full overflow-scroll">
        {#if filteredActions.length > 0}
          {#each filteredActions as action (action.title)}
            {@const {
              title,
              description,
              icon: Icon,
              callback,
              shortcut,
            } = action}
            <CommandItem
              value={title}
              {command}
              onSelect={() => {
                callback();
                query = "";
                open = false;
              }}
            >
              <Icon class="mr-2 h-4 w-4" />
              {description}
              {#if shortcut}
                <ShortcutBadge class="ml-auto" {shortcut} />
              {/if}
            </CommandItem>
          {/each}
        {:else}
          <div class="text-center">No results found.</div>
        {/if}
      </div>
    {/snippet}
  </Command>
</Modal>
