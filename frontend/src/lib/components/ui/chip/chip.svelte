<script lang="ts">
  import type { ChipProps } from ".";

  let { title, description = "" }: ChipProps = $props();
</script>

<div
  class="text-nowrap rounded border bg-secondary px-1 text-xs text-secondary-foreground max-sm:hidden"
  title={description}
>
  {title}
</div>
