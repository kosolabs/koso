<script lang="ts">
  import { X } from "lucide-svelte";
  import type { ChipProps } from ".";

  let { title, description = "", onClick, onDelete }: ChipProps = $props();
</script>

<div
  class="bg-secondary text-secondary-foreground flex items-center gap-1 rounded border px-1 text-xs text-nowrap"
  title={description}
>
  {#if onClick}
    <button aria-label="Jump to {title}" onclick={onClick}>
      {title}
    </button>
  {:else}
    {title}
  {/if}

  {#if onDelete}
    <button
      aria-label="Delete {title}"
      class="hover:border-primary-foreground rounded border border-transparent active:scale-95"
      onclick={onDelete}
    >
      <X size={12} />
    </button>
  {/if}
</div>
