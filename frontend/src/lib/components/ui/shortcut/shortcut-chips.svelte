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

<div class={cn("text-foreground flex gap-1 p-1", classes)}>
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
