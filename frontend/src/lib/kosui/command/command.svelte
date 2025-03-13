<script module lang="ts">
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import type { Command, CommandItem } from ".";
  import { Shortcut } from "../shortcut";
  import type { ClassName } from "../utils";

  export type CommandProps = {
    input: Snippet<[Command]>;
    content: Snippet<[Command]>;
  } & ClassName;
</script>

<script lang="ts">
  let { input, content, class: className }: CommandProps = $props();

  let el: HTMLDivElement | undefined = $state();
  let value: string = $state("");
  let items: Record<string, CommandItem> = {};
  let focused: string | undefined = $state();

  export function setInputValue(newValue: string) {
    if (value !== newValue) {
      value = newValue;
      if (value === "") {
        blur();
      } else {
        const itemIds = getItemIds();
        if (itemIds.length > 0) {
          focus(itemIds[0]);
        }
      }
    }
  }

  export function register(item: CommandItem) {
    items[item.getId()] = item;
  }

  export function unregister(item: CommandItem) {
    delete items[item.getId()];
  }

  function getItemIds(): string[] {
    if (!el) return [];
    return Array.from(el.getElementsByTagName("button"))
      .filter((button) => button.role === "option")
      .map((button) => button.id)
      .filter((id) => id in items);
  }

  export function focus(id: string) {
    if (focused && focused in items) {
      items[focused].blur();
    }
    focused = id;
    items[focused].focus();
  }

  function blur() {
    if (focused && focused in items) {
      items[focused].blur();
    }
    focused = undefined;
  }

  export function handleKeyDown(event: KeyboardEvent) {
    if (!items) return;
    const itemIds = getItemIds();
    if (Shortcut.ARROW_UP.matches(event)) {
      if (itemIds.length !== 0) {
        if (!focused) {
          focus(itemIds[itemIds.length - 1]);
        } else {
          const index = itemIds.indexOf(focused);
          focus(itemIds[(index - 1 + itemIds.length) % itemIds.length]);
        }
      }
      event.preventDefault();
      event.stopImmediatePropagation();
    } else if (Shortcut.ARROW_DOWN.matches(event)) {
      if (itemIds.length !== 0) {
        if (!focused) {
          focus(itemIds[0]);
        } else {
          const index = itemIds.indexOf(focused);
          focus(itemIds[(index + 1) % itemIds.length]);
        }
      }
      event.preventDefault();
      event.stopImmediatePropagation();
    } else if (Shortcut.ENTER.matches(event)) {
      if (focused && focused in items) {
        items[focused].select();
        event.preventDefault();
        event.stopImmediatePropagation();
      }
    }
  }

  const self: Command = {
    setInputValue,
    register,
    unregister,
    handleKeyDown,
    focus,
  };
</script>

<div bind:this={el} class={twMerge("p-1", className)}>
  {@render input(self)}
  <hr class="-mx-1 my-1" />
  {@render content(self)}
</div>
