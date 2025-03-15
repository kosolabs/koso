<script module lang="ts">
  import { type Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import type { Autocomplete, AutocompleteItem } from ".";
  import { baseClasses, type Variants } from "../base";
  import { mergeProps } from "../merge-props";
  import { uid, type ClassName, type ElementRef } from "../utils";

  export type AutocompleteItemProps = {
    onSelect?: () => void;
    autocomplete: Autocomplete;
    children: Snippet<[]>;
  } & ElementRef &
    ClassName &
    Variants &
    HTMLAttributes<HTMLDivElement>;
</script>

<script lang="ts">
  let {
    onSelect,
    autocomplete,
    children,
    el = $bindable(),
    class: className,
    variant = "plain",
    color = "secondary",
    shape = "rounded",
    ...restProps
  }: AutocompleteItemProps = $props();

  let id: string = $state(uid({ prefix: "autocomplete" }));
  let focused: boolean = $state(false);

  export function getId() {
    return id;
  }

  export function focus() {
    focused = true;
  }

  export function blur() {
    focused = false;
  }

  export function select() {
    onSelect?.();
  }

  const self: AutocompleteItem = { getId, focus, blur, select };

  $effect(() => {
    autocomplete.register(self);
    return () => autocomplete.unregister(self);
  });
</script>

<div
  {id}
  bind:this={el}
  role="option"
  aria-selected={focused}
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "aria-selected:bg-m3-secondary/15 flex w-full items-center gap-1 px-2 py-1 text-left text-sm focus:ring-0",
    className,
  )}
  {...mergeProps(restProps, {
    onclick: select,
    onmouseenter: () => autocomplete.focus(id),
  })}
>
  {@render children()}
</div>
