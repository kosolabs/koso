<script module lang="ts">
  import { match } from "$lib/utils";
  import { Icon, SearchIcon } from "lucide-svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { Command, CommandDivider, CommandInput, CommandItem } from ".";
  import { events } from "..";
  import { Modal } from "../modal";
  import { Shortcut, ShortcutBadge } from "../shortcut";

  export type Action = {
    callback: () => void;
    title: string;
    description: string;
    icon: typeof Icon;
    enabled: () => boolean;
    shortcut?: Shortcut;
  };

  let open: boolean = $state(false);
  let query: string = $state("");

  export const actions: Set<Action> = new SvelteSet();
  const shortcuts: Record<string, Action> = {};

  const filteredActions = $derived(
    Array.from(actions).filter(
      (action) =>
        action.enabled() &&
        (match(action.title, query) || match(action.description, query)),
    ),
  );

  export function show() {
    open = true;
  }

  export function close() {
    open = false;
  }

  export function register(action: Action) {
    actions.add(action);
    if (action.shortcut) {
      shortcuts[action.shortcut.toString()] = action;
    }
    return () => unregister(action);
  }

  export function unregister(action: Action) {
    actions.delete(action);
    if (action.shortcut) {
      delete shortcuts[action.shortcut.toString()];
    }
  }

  function handleSelect(action: Action) {
    action.callback();
    close();
  }

  function handleKeyDown(event: KeyboardEvent) {
    const action = shortcuts[Shortcut.fromEvent(event).toString()];
    if (action && action.enabled()) {
      action.callback();
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  }
</script>

<script lang="ts">
  $effect(() => {
    return events.on("keydown", handleKeyDown);
  });
</script>

<Modal
  bind:open
  onoutroend={() => (query = "")}
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
          {@const { description, shortcut, enabled, ...restProps } = action}
          <CommandItem
            onSelect={() => handleSelect(action)}
            disabled={!enabled()}
            {...restProps}
          >
            <action.icon class="mr-2 h-4 w-4" />
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
  </Command>
</Modal>
