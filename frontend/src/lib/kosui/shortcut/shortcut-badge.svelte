<script module lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import { twMerge, type ClassNameValue } from "tailwind-merge";
  import { baseClasses } from "../base";
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
        baseClasses({ variant: "elevated", color: "secondary" }),
        "flex h-5 min-w-5 items-center justify-center rounded border p-1 text-xs",
        badgeClass,
      )}
    >
      {symbol}
    </div>
  {/each}
</div>
