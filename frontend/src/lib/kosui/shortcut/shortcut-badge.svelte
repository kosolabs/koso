<script module lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import { twMerge, type ClassNameValue } from "tailwind-merge";
  import { Box } from "../box";
  import { noop, type ClassName, type ElementRef } from "../utils";
  import type { Shortcut } from "./shortcut";

  export type ShortcutBadgeProps = {
    shortcut: Shortcut;
    badgeClass?: ClassNameValue;
  } & ElementRef &
    ClassName &
    HTMLInputAttributes;
</script>

<script lang="ts">
  let {
    shortcut,
    el = $bindable(),
    ref = noop,
    class: className,
    badgeClass,
  }: ShortcutBadgeProps = $props();
</script>

<div
  bind:this={el}
  use:ref
  class={twMerge("text-m3-on-surface flex gap-1", className)}
>
  {#each shortcut as symbol}
    <Box
      variant="elevated"
      color="secondary"
      centered
      class={twMerge("h-5 min-w-5 rounded border p-1 text-xs", badgeClass)}
    >
      {symbol}
    </Box>
  {/each}
</div>
