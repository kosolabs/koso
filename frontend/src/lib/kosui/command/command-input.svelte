<script module lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import type { Command } from ".";
  import { baseClasses, type Variants } from "../base";
  import type { ClassName, ElementRef } from "../utils";

  export type CommandInputProps = { command: Command } & ElementRef &
    ClassName &
    Variants &
    HTMLInputAttributes;
</script>

<script lang="ts">
  import { twMerge } from "tailwind-merge";

  let {
    value = $bindable(""),
    command,
    class: className,
    variant = "plain",
    color = "secondary",
    shape = "rounded",
    ...restProps
  }: CommandInputProps = $props();

  $effect(() => command.setInputValue(value));
</script>

<input
  bind:value
  onkeydown={command.handleKeyDown}
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "w-full p-1 focus-visible:outline-hidden",
    className,
  )}
  {...restProps}
/>
