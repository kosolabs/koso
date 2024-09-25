<script lang="ts">
  import type { ChipProps } from ".";

  let { title, description = "" }: ChipProps = $props();
</script>

<div
  class="text-nowrap rounded border bg-primary-foreground px-1 text-xs max-sm:hidden"
  title={description}
>
  {title}
</div>
