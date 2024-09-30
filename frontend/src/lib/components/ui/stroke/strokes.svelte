<script lang="ts">
  import type { KeyBinding } from "$lib/key-binding";
  import { cn } from "$lib/utils";
  import { Stroke } from ".";

  type Props = {
    binding: KeyBinding;
    class?: string;
  };
  const { binding, class: classes }: Props = $props();
</script>

<div class={cn("flex gap-1 p-1", classes)}>
  {#if binding.ctrlKey}
    <Stroke>⌃</Stroke>
  {/if}
  {#if binding.shiftKey}
    <Stroke>⇧</Stroke>
  {/if}
  {#if binding.altKey}
    <Stroke>⌥</Stroke>
  {/if}
  {#if binding.metaKey}
    <Stroke>⌘</Stroke>
  {/if}
  <Stroke>
    {binding.key.length === 1 ? binding.key.toUpperCase() : binding.key}
  </Stroke>
</div>
