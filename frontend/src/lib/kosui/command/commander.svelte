<script module lang="ts">
  import { match } from "$lib/utils";
  import { Icon, Terminal } from "lucide-svelte";
  import { onMount, untrack } from "svelte";
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

  export type CommanderProps = {
    title?: string;
    description?: string;
    icon?: typeof Icon;
    enabled?: () => boolean;
    shortcut?: Shortcut;
  };

  // TODO: Add a name field
  export type Action = {
    callback: () => void;
    title: string;
    description: string;
    icon: typeof Icon;
    enabled: () => boolean;
    shortcut?: Shortcut;
  };

  class Registry {
    #actions: Record<string, Action> = $state({});
    #shortcuts: Record<string, Action> = {};

    get actions(): Action[] {
      return Object.values(this.#actions);
    }

    get(title: string): Action | undefined {
      return this.#actions[title];
    }

    getByShortcut(shortcut: Shortcut): Action | undefined {
      return this.#shortcuts[shortcut.toString()];
    }

    call(title: string) {
      const action = this.get(title);
      if (!action) {
        throw new Error(`No action named "${title}" is registered`);
      }
      return action;
    }

    register(...actions: Action[]) {
      for (const action of actions) {
        untrack(() => {
          if (action.title in this.#actions) {
            throw new Error(`${action.title} is already registered`);
          }
        });
        this.#actions[action.title] = action;
        if (action.shortcut) {
          if (action.shortcut.toString() in this.#shortcuts) {
            throw new Error(`${action.shortcut} is already registered`);
          }
          this.#shortcuts[action.shortcut.toString()] = action;
        }
      }
      return () => this.unregister(...actions);
    }

    unregister(...actions: Action[]) {
      for (const action of actions) {
        delete this.#actions[action.title];
        if (action.shortcut) {
          delete this.#shortcuts[action.shortcut.toString()];
        }
      }
    }
  }

  export const command = new Registry();
</script>

<script lang="ts">
  let {
    title = "Command Palette",
    description = "Show the command palette",
    icon = Terminal,
    enabled = () => true,
    shortcut = new Shortcut({ key: "p", shift: true, meta: true }),
  }: CommanderProps = $props();

  let open: boolean = $state(false);
  let query: string = $state("");

  const filteredActions = $derived(
    command.actions.filter(
      (action) =>
        action.enabled() &&
        (match(action.title, query) || match(action.description, query)),
    ),
  );

  function handleSelect(action: Action) {
    action.callback();
    open = false;
  }

  function handleKeyDown(event: KeyboardEvent) {
    const action = command.getByShortcut(Shortcut.fromEvent(event));
    if (action && action.enabled()) {
      action.callback();
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  }

  $effect(() => {
    return events.on("keydown", handleKeyDown);
  });

  onMount(() => {
    return command.register({
      callback: () => (open = true),
      title,
      description,
      icon,
      enabled,
      shortcut,
    });
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
