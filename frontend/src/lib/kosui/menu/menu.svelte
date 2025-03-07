<script module lang="ts">
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import { mergeComponentProps } from "../merge-props";
  import { Popover, type PopoverProps } from "../popover";
  import { Shortcut } from "../shortcut";
  import { TypingBuffer, uid, type ClassName } from "../utils";

  export type MenuTriggerProps = {
    ref: (el: HTMLElement) => void;
    onclick: (event: MouseEvent) => void;
    onkeydown: (event: KeyboardEvent) => void;
    "aria-controls": string;
    "aria-haspopup": "menu";
    "aria-expanded": boolean;
  };

  export type MenuItemProps = {
    ref: (el: HTMLElement) => void;
    unref: (el: HTMLElement) => void;
    onclick: (event: MouseEvent) => void;
    onmouseenter: (event: MouseEvent) => void;
    onfocus: (event: FocusEvent) => void;
    onkeydown: (event: KeyboardEvent) => void;
    role: "menuitem";
  };

  export type MenuProps = {
    uncontrolled?: boolean;
    trigger?: Snippet<[MenuTriggerProps]>;
    content: Snippet<[MenuItemProps]>;
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

  let menuId = "menu-" + uid();
  let menuItems: HTMLElement[] = [];
  let focusedItem: HTMLElement | undefined = $state(undefined);
  let buffer = new TypingBuffer();

  function focusAnchor() {
    anchorEl?.focus();
  }

  function close() {
    if (uncontrolled) return;
    open = false;
    focusAnchor();
  }

  function focus(menuItem?: HTMLElement) {
    if (!menuItem) return;
    focusedItem = menuItem;
    focusedItem.focus();
  }

  function blur() {
    focusedItem?.blur();
    focusedItem = undefined;
  }

  function register(menuItem?: HTMLElement) {
    if (!menuItem) return;
    menuItems.push(menuItem);
  }

  function unregister(menuItem?: HTMLElement) {
    if (!menuItem) return;
    const index = menuItems.indexOf(menuItem);
    if (index !== -1) {
      menuItems.splice(index, 1);
    }
  }

  function handleSelect() {
    close();
  }

  function handleMouseEnter(event: MouseEvent) {
    if (event.target instanceof HTMLElement) {
      focus(event.target);
    }
  }

  function handleFocus(event: FocusEvent) {
    if (event.target instanceof HTMLElement) {
      focus(event.target);
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (!menuItems) return;
    if (
      Shortcut.ARROW_UP.matches(event) ||
      Shortcut.TAB_BACKWARD.matches(event)
    ) {
      if (!focusedItem) {
        focus(menuItems[menuItems.length - 1]);
      } else {
        let activeIndex = menuItems.indexOf(focusedItem);
        activeIndex = (activeIndex - 1 + menuItems.length) % menuItems.length;
        focus(menuItems[activeIndex]);
      }
      event.preventDefault();
      event.stopImmediatePropagation();
    } else if (
      Shortcut.ARROW_DOWN.matches(event) ||
      Shortcut.TAB_FORWARD.matches(event)
    ) {
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
    if (!open) {
      menuItems = [];
    }
  });
</script>

{#if trigger}
  {@render trigger({
    ref: (ref) => (anchorEl = ref),
    onclick: () => (open = !open),
    onkeydown: (event: KeyboardEvent) => {
      if (Shortcut.ENTER.matches(event) || Shortcut.SPACE.matches(event)) {
        event.stopImmediatePropagation();
      }
    },
    "aria-controls": menuId,
    "aria-haspopup": "menu",
    "aria-expanded": open,
  })}
{/if}

<Popover
  bind:open
  id={menuId}
  role="menu"
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "bg-m3-surface-container-highest max-h-[40%] border p-1 shadow",
    className,
  )}
  {placement}
  {anchorEl}
  {...mergeComponentProps(Popover, restProps, {
    onmouseleave: blur,
    onKeydownWhileOpen: handleKeyDown,
  })}
>
  {@render content({
    ref: register,
    unref: unregister,
    onclick: handleSelect,
    onmouseenter: handleMouseEnter,
    onfocus: handleFocus,
    onkeydown: (event: KeyboardEvent) => {
      if (Shortcut.ENTER.matches(event) || Shortcut.SPACE.matches(event)) {
        event.stopImmediatePropagation();
      }
    },
    role: "menuitem",
  })}
</Popover>
