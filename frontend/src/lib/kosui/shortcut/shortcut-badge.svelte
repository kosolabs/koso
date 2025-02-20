<script module lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import { tv, type ClassValue, type VariantProps } from "tailwind-variants";
  import type { ClassName, ElementRef } from "../utils";
  import type { Shortcut } from "./shortcut";

  export const shortcutBadgeVariants = tv({
    slots: {
      base: "text-m3-on-surface flex gap-1",
      badge:
        "bg-m3-surface-container text-m3-on-surface shadow-m3-shadow/10 border-m3-outline flex h-5 min-w-5 items-center justify-center rounded border p-1 text-xs shadow",
    },
  });

  export type ShortcutBadgeVariants = VariantProps<
    typeof shortcutBadgeVariants
  >;

  export type ShortcutBadgeProps = {
    shortcut: Shortcut;
    badgeClass?: ClassValue;
  } & ClassName &
    ShortcutBadgeVariants &
    ElementRef &
    HTMLInputAttributes;
</script>

<script lang="ts">
  const {
    shortcut,
    class: className,
    badgeClass,
  }: ShortcutBadgeProps = $props();
  const { base, badge } = shortcutBadgeVariants();
</script>

<div class={base({ className })}>
  {#each shortcut as symbol}
    <div class={badge({ className: badgeClass })}>
      {symbol}
    </div>
  {/each}
</div>
