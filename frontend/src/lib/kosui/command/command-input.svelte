<script module lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import { mergeProps } from "../merge-props";
  import type { ClassName, ElementRef } from "../utils";
  import { getCommandContext } from "./command-context.svelte";

  export type CommandInputProps = {} & ElementRef &
    ClassName &
    Variants &
    HTMLInputAttributes;
</script>

<script lang="ts">
  let {
    value = $bindable(""),
    class: className,
    variant = "plain",
    color = "secondary",
    shape = "rounded",
    ...restProps
  }: CommandInputProps = $props();

  const ctx = getCommandContext();
  ctx.bindInput(
    () => value,
    (newValue) => (value = newValue),
  );

  function handleKeyDown(event: KeyboardEvent) {
    ctx.handleKeyDown(event);
  }
</script>

<input
  bind:value={ctx.input}
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "w-full p-1 focus-visible:outline-hidden",
    className,
  )}
  {...mergeProps({ onkeydown: handleKeyDown }, restProps)}
/>
