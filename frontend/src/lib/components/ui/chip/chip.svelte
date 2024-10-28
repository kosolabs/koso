<script lang="ts">
  import type { ChipProps } from ".";
  import { Button } from "$lib/components/ui/button";

  let { title, description = "", onClick }: ChipProps = $props();
</script>

<div
  class="text-nowrap rounded border bg-secondary px-1 text-xs text-secondary-foreground max-sm:hidden"
  title={description}
>
  {#if onClick}
    <Button onclick={onClick}>
      {title}
    </Button>
  {:else}
    {title}
  {/if}
</div>
