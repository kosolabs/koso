<script module lang="ts">
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import type { Menu } from ".";
  import { events } from "..";
  import { baseClasses, type Variants } from "../base";
  import { mergeProps } from "../merge-props";
  import { Popover, type PopoverProps } from "../popover";
  import { Shortcut } from "../shortcut";
  import { TypingBuffer, type ClassName } from "../utils";

  export type MenuProps = {
    uncontrolled?: boolean;
    trigger?: Snippet<
      [{ ref: (el: HTMLElement) => void; onclick: () => void }]
    >;
    content: Snippet<[Menu]>;
  } & ClassName &
    Variants &
    Omit<PopoverProps, "children">;
</script>

<script lang="ts">
  let {
    uncontrolled = false,
    trigger,
    content,
    anchorEl,
    class: className,
    variant = "elevated",
    color = "primary",
    shape = "rounded",
    open = $bindable(false),
    placement = "bottom",
    ...restProps
  }: MenuProps = $props();

  let menuItems: HTMLElement[] = [];
  let focusedItem: HTMLElement | undefined = $state(undefined);
  let buffer = new TypingBuffer();

  function focusAnchor() {
    anchorEl?.focus();
  }

  export function close() {
    if (uncontrolled) return;
    open = false;
    focusAnchor();
  }

  export function focus(menuItem?: HTMLElement) {
    if (!menuItem) return;
    focusedItem = menuItem;
    focusedItem.focus();
  }

  function blur() {
    focusedItem?.blur();
    focusedItem = undefined;
  }

  export function register(menuItem?: HTMLElement) {
    if (!menuItem) return;
    menuItems.push(menuItem);
  }

  export function unregister(menuItem?: HTMLElement) {
    if (!menuItem) return;
    const index = menuItems.indexOf(menuItem);
    if (index !== -1) {
      menuItems.splice(index, 1);
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (!menuItems) return;
    if (Shortcut.ARROW_UP.matches(event)) {
      if (!focusedItem) {
        focus(menuItems[menuItems.length - 1]);
      } else {
        let activeIndex = menuItems.indexOf(focusedItem);
        activeIndex = (activeIndex - 1 + menuItems.length) % menuItems.length;
        focus(menuItems[activeIndex]);
      }
      event.preventDefault();
      event.stopImmediatePropagation();
    } else if (Shortcut.ARROW_DOWN.matches(event)) {
      if (!focusedItem) {
        focus(menuItems[0]);
      } else {
        let activeIndex = menuItems.indexOf(focusedItem);
        activeIndex = (activeIndex + 1) % menuItems.length;
        focus(menuItems[activeIndex]);
      }
      event.preventDefault();
      event.stopImmediatePropagation();
    } else if (Shortcut.HOME.matches(event)) {
      focus(menuItems[0]);
      event.preventDefault();
      event.stopImmediatePropagation();
    } else if (Shortcut.END.matches(event)) {
      focus(menuItems[menuItems.length - 1]);
      event.preventDefault();
      event.stopImmediatePropagation();
    } else if (Shortcut.ENTER.matches(event)) {
      event.stopImmediatePropagation();
    } else if (Shortcut.isChar(event)) {
      const prefix = buffer.append(event.key);
      const matchedItem = menuItems.find((menuItem) =>
        (menuItem.textContent?.trim().toLowerCase() ?? "").startsWith(
          prefix.toLowerCase(),
        ),
      );
      focus(matchedItem);
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  }

  $effect(() => {
    if (open) {
      events.on("keydown", handleKeyDown);
    } else {
      events.remove("keydown", handleKeyDown);
    }

    return () => events.remove("keydown", handleKeyDown);
  });

  const self: Menu = { close, focus, register, unregister };
</script>

{#if trigger}
  {@render trigger({
    ref: (ref) => (anchorEl = ref),
    onclick: () => (open = !open),
  })}
{/if}

<Popover
  bind:open
  role="menu"
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "bg-m3-surface-container-highest max-h-[40%] border p-1 shadow",
    className,
  )}
  {...mergeProps(restProps, {
    onmouseleave: blur,
    placement,
    anchorEl,
  })}
>
  {@render content(self)}
</Popover>
