<script module lang="ts">
  import { type Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import type { Menu } from ".";
  import { baseClasses, type Variants } from "../base";
  import type { ClassName, ElementRef } from "../utils";

  export type MenuItemProps = {
    menuRef: Menu;
    onSelect?: () => void;
    closeOnSelect?: boolean;
    children: Snippet<[]>;
  } & ElementRef &
    ClassName &
    Variants &
    Omit<HTMLButtonAttributes, "type" | "value">;
</script>

<script lang="ts">
  let {
    menuRef,
    onSelect,
    children,
    closeOnSelect = true,
    ref = $bindable(),
    class: className,
    variant = "plain",
    color = "secondary",
    shape = "rounded",
    ...restProps
  }: MenuItemProps = $props();

  function handleSelect() {
    if (closeOnSelect) {
      menuRef.close();
    }
    onSelect?.();
  }

  $effect(() => {
    if (ref) {
      menuRef.register(ref);
      return () => menuRef.unregister(ref);
    }
  });
</script>

<button
  bind:this={ref}
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "block w-full px-2 py-1 focus-visible:ring-0",
    className,
  )}
  onclick={handleSelect}
  onmouseenter={() => menuRef.focus(ref)}
  onfocus={() => menuRef.focus(ref)}
  {...restProps}
>
  {@render children()}
</button>
