<script lang="ts">
  import type { KeyBinding } from "$lib/key-binding";
  import { Stroke } from ".";

  type Props = {
    binding: KeyBinding;
  };
  const { binding }: Props = $props();
</script>

<div class="flex gap-1 p-1">
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
