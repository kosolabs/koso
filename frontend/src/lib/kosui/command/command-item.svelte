<script module lang="ts">
  import { type Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { Command, CommandItem } from ".";
  import { baseClasses, type Variants } from "../base";
  import { uid, type ClassName, type ElementRef } from "../utils";

  export type CommandItemProps = {
    onSelect?: () => void;
    command: Command;
    children: Snippet<[]>;
  } & ElementRef &
    ClassName &
    Variants &
    HTMLAttributes<HTMLDivElement>;
</script>

<script lang="ts">
  import { mergeProps } from "../merge-props";

  let {
    onSelect,
    command,
    children,
    el = $bindable(),
    class: className,
    variant = "plain",
    color = "secondary",
    shape = "rounded",
    ...restProps
  }: CommandItemProps = $props();

  let id: string = $state(uid({ prefix: "command" }));
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

  const self: CommandItem = { getId, focus, blur, select };

  $effect(() => {
    command.register(self);
    return () => command.unregister(self);
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
    onmouseenter: () => command.focus(id),
  })}
>
  {@render children()}
</div>
