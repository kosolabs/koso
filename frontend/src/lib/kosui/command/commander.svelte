<script module lang="ts">
  import { match } from "$lib/utils";
  import { Icon } from "lucide-svelte";
  import {
    Command,
    CommandContent,
    CommandDivider,
    CommandItem,
    CommandSearch,
  } from ".";
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

  export const registry: Record<string, Action> = $state({});
  const shortcuts: Record<string, Action> = {};

  const filteredActions = $derived(
    actions().filter(
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

  export function register(...actions: Action[]) {
    for (const action of actions) {
      if (action.title in registry) {
        throw new Error(`${action.title} is already registered`);
      }
      registry[action.title] = action;
      if (action.shortcut) {
        if (action.shortcut.toString() in shortcuts) {
          throw new Error(`${action.shortcut} is already registered`);
        }
        shortcuts[action.shortcut.toString()] = action;
      }
    }
    return () => unregister(...actions);
  }

  export function unregister(...actions: Action[]) {
    for (const action of actions) {
      delete registry[action.title];
      if (action.shortcut) {
        delete shortcuts[action.shortcut.toString()];
      }
    }
  }

  export function get(title: string): Action | undefined {
    return registry[title];
  }

  export function call(title: string) {
    const action = get(title);
    if (!action) {
      throw new Error(`No action named "${title}" is registered`);
    }
    return action;
  }

  export function actions(): Action[] {
    return Object.values(registry);
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
  <Command>
    <CommandSearch bind:value={query} />
    <CommandDivider />
    <CommandContent>
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
    </CommandContent>
  </Command>
</Modal>
