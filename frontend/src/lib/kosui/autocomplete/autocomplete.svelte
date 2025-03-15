<script module lang="ts">
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import type { Autocomplete, AutocompleteItem } from ".";
  import { baseClasses, type Variants } from "../base";
  import { Popover, type PopoverProps } from "../popover";
  import { Shortcut } from "../shortcut";
  import type { ClassName } from "../utils";

  export type AutocompleteProps = {
    input: Snippet<[Autocomplete]>;
    content: Snippet<[Autocomplete]>;
    showCompletions?: boolean;
  } & ClassName &
    Variants &
    Omit<PopoverProps, "children">;
</script>

<script lang="ts">
  let {
    input,
    content,
    showCompletions = $bindable(false),
    class: className,
    variant = "elevated",
    color = "primary",
    shape = "rounded",
    ...restProps
  }: AutocompleteProps = $props();

  let anchorEl: HTMLElement | undefined = $state();
  let popoverEl: HTMLElement | undefined = $state();
  let value: string = $state("");
  let items: Record<string, AutocompleteItem> = {};
  let focused: string | undefined = $state();

  export function setAnchorEl(newAnchorEl: HTMLElement) {
    if (anchorEl !== newAnchorEl) {
      anchorEl = newAnchorEl;
    }
  }

  export function setInputValue(newValue: string) {
    if (value !== newValue) {
      value = newValue;
    }
  }

  export function register(item: AutocompleteItem) {
    items[item.getId()] = item;
  }

  export function unregister(item: AutocompleteItem) {
    delete items[item.getId()];
  }

  function getItemIds(): string[] {
    if (!popoverEl) return [];
    return Array.from(popoverEl.getElementsByTagName("div"))
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

  const self: Autocomplete = {
    setInputValue,
    setAnchorEl,
    register,
    unregister,
    handleKeyDown,
    focus,
  };
</script>

{@render input(self)}

<Popover
  bind:el={popoverEl}
  bind:open={showCompletions}
  role="menu"
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "bg-m3-surface-container-highest max-h-[40%] border p-1 shadow",
    className,
  )}
  {anchorEl}
  placement="bottom"
  {...restProps}
>
  {@render content(self)}
</Popover>
