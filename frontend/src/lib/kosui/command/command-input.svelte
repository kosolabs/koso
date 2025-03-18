<script module lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
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
  ctx.bind(
    () => value,
    (newValue) => (value = newValue),
  );
</script>

<input
  bind:value={ctx.value}
  onkeydown={(event) => ctx.handleKeyDown(event)}
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "w-full p-1 focus-visible:outline-hidden",
    className,
  )}
  {...restProps}
/>
