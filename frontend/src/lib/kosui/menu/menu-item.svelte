<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import type { Menu } from ".";
  import { baseClasses, type Variants } from "../base";
  import type { ClassName } from "../utils";

  export type MenuItemProps = {
    menuRef: Menu;
    onSelect?: () => void;
    closeOnSelect?: boolean;
    children: Snippet<[]>;
  } & ClassName &
    Variants &
    Omit<HTMLButtonAttributes, "type" | "value">;
</script>

<script lang="ts">
  let {
    menuRef,
    onSelect,
    children,
    closeOnSelect = true,
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
</script>

<button
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "block px-2 py-1",
    className,
  )}
  onclick={handleSelect}
  {...restProps}
>
  {@render children()}
</button>
