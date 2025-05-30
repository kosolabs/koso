<script module lang="ts">
  import { match } from "$lib/utils";
  import {
    Action,
    Command,
    CommandContent,
    CommandDivider,
    CommandItem,
    CommandSearch,
    Registry,
  } from ".";
  import { events } from "..";
  import { Modal } from "../modal";
  import { Shortcut, ShortcutBadge } from "../shortcut";

  export type CommanderProps = {
    open?: boolean;
    query?: string;
    command: Registry;
  };
</script>

<script lang="ts">
  let {
    open = $bindable(false),
    query = $bindable(""),
    command,
  }: CommanderProps = $props();

  const actions = $derived(
    command.actions.filter(
      (action) =>
        action.enabled() &&
        (match(action.id, query) ||
          match(action.title, query) ||
          match(action.description, query)),
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
      {#if actions.length > 0}
        {#each actions as action (action.title)}
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
