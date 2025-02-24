<script module lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import { twMerge, type ClassNameValue } from "tailwind-merge";
  import type { ClassName, ElementRef } from "../utils";
  import type { Shortcut } from "./shortcut";

  export type ShortcutBadgeProps = {
    shortcut: Shortcut;
    badgeClass?: ClassNameValue;
  } & ElementRef &
    ClassName &
    HTMLInputAttributes;
</script>

<script lang="ts">
  const {
    shortcut,
    class: className,
    badgeClass,
  }: ShortcutBadgeProps = $props();
</script>

<div class={twMerge("text-m3-on-surface flex gap-1", className)}>
  {#each shortcut as symbol}
    <div
      class={twMerge(
        "bg-m3-surface-container text-m3-on-surface shadow-m3-shadow/10 border-m3-outline flex h-5 min-w-5 items-center justify-center rounded border p-1 text-xs shadow",
        badgeClass,
      )}
    >
      {symbol}
    </div>
  {/each}
</div>
