<script lang="ts">
  import { X } from "lucide-svelte";
  import type { ChipProps } from ".";

  let { title, description = "", onClick, onDelete }: ChipProps = $props();
</script>

<div
  class="flex items-center gap-1 text-nowrap rounded border bg-secondary px-1 text-xs text-secondary-foreground max-sm:hidden"
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
      class="rounded border border-transparent hover:border-primary-foreground active:scale-95"
      onclick={onDelete}
    >
      <X size={12} />
    </button>
  {/if}
</div>
