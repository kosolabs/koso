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
    <Button
      onclick={onClick}
      variant="link"
      class="h-1 p-0 text-xs text-secondary-foreground"
    >
      {title}
    </Button>
  {:else}
    {title}
  {/if}
</div>
