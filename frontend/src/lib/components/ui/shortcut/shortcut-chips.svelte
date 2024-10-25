<script lang="ts">
  import type { Shortcut } from "$lib/shortcuts";
  import { cn } from "$lib/utils";
  import { ShortcutChip } from ".";

  type Props = {
    shortcut: Shortcut;
    class?: string;
  };
  const { shortcut, class: classes }: Props = $props();
</script>

<div class={cn("flex gap-1 p-1 text-foreground", classes)}>
  {#if shortcut.ctrl}
    <ShortcutChip>⌃</ShortcutChip>
  {/if}
  {#if shortcut.shift}
    <ShortcutChip>⇧</ShortcutChip>
  {/if}
  {#if shortcut.alt}
    <ShortcutChip>⌥</ShortcutChip>
  {/if}
  {#if shortcut.meta}
    <ShortcutChip>⌘</ShortcutChip>
  {/if}
  <ShortcutChip>
    {shortcut.toChar()}
  </ShortcutChip>
</div>
