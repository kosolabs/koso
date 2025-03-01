<script module lang="ts">
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import type { Menu } from ".";
  import { events } from "..";
  import { baseClasses, type Variants } from "../base";
  import { Popover, type PopoverProps } from "../popover";
  import { Shortcut } from "../shortcut";
  import type { ClassName } from "../utils";

  export type MenuProps = {
    uncontrolled?: boolean;
    content: Snippet<[Menu]>;
  } & ClassName &
    Variants &
    Omit<PopoverProps, "children">;
</script>

<script lang="ts">
  let {
    uncontrolled = false,
    content,
    class: className,
    variant = "elevated",
    color = "primary",
    shape = "rounded",
    open = $bindable(),
    placement = "bottom",
    ...restProps
  }: MenuProps = $props();

  const menuItems: HTMLElement[] = [];
  let activeIndex = $state(-1);

  export function close() {
    if (uncontrolled) return;
    open = false;
  }

  export function focus(menuItem?: HTMLElement) {
    if (!menuItem) return;
    activeIndex = menuItems.indexOf(menuItem);
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
    if (Shortcut.ARROW_UP.matches(event)) {
      activeIndex = (activeIndex - 1 + menuItems.length) % menuItems.length;
    } else if (Shortcut.ARROW_DOWN.matches(event)) {
      activeIndex = (activeIndex + 1) % menuItems.length;
    } else if (Shortcut.HOME.matches(event)) {
      activeIndex = 0;
    } else if (Shortcut.END.matches(event)) {
      activeIndex = menuItems.length - 1;
    } else {
      return;
    }
    event.preventDefault();
    event.stopImmediatePropagation();
  }

  $effect(() => {
    if (open) {
      events.on("keydown", handleKeyDown);
    } else {
      events.remove("keydown", handleKeyDown);
    }

    return () => events.remove("keydown", handleKeyDown);
  });

  $effect(() => {
    menuItems[activeIndex]?.focus();
  });

  const self: Menu = { close, focus, register, unregister };
</script>

<Popover
  bind:open
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "bg-m3-surface-container-highest max-h-[40%] border p-1 shadow",
    className,
  )}
  {placement}
  {...restProps}
>
  {@render content(self)}
</Popover>
