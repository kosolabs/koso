<script module lang="ts">
  import { match } from "$lib/utils";
  import { Icon, SearchIcon } from "lucide-svelte";
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

  export const actions: Record<string, Action> = $state({});
  const shortcuts: Record<string, Action> = {};

  const filteredActions = $derived(
    Object.values(actions).filter(
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
    actions[action.title] = action;
    if (action.shortcut) {
      shortcuts[action.shortcut.toString()] = action;
    }
    return () => unregister(action);
  }

  export function unregister(action: Action) {
    delete actions[action.title];
    if (action.shortcut) {
      delete shortcuts[action.shortcut.toString()];
    }
  }

  export function call(title: string) {
    if (title in actions) {
      actions[title].callback();
    } else {
      throw new Error(`No action with "${title}" is registered`);
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
  class="bg-m3-surface-container h-[min(60%,48em)] w-[min(calc(100%-1em),36em)] rounded-lg p-0"
>
  <Command class="flex h-full flex-col">
    <div class="flex items-center gap-2 px-2">
      <SearchIcon size={16} />
      <CommandInput
        autofocus
        bind:value={query}
        placeholder="Search..."
        class="h-10"
      />
    </div>
    <CommandDivider />
    <div class="h-full overflow-scroll">
      {#if filteredActions.length > 0}
        {#each filteredActions as action (action.title)}
          {@const { description, shortcut, ...restProps } = action}
          <CommandItem onSelect={() => handleSelect(action)} {...restProps}>
            <action.icon size={14} class="mr-2" />
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
