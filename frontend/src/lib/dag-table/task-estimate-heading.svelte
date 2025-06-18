<script lang="ts">
  import { Button } from "$lib/kosui/button";
  import { Tooltip } from "$lib/kosui/tooltip";
  import { CalendarDays } from "@lucide/svelte";
</script>

<Tooltip class="flex max-h-2/5 max-w-3/5" click arrow>
  {#snippet trigger({ ...restProps })}
    <Button
      variant="plain"
      color="primary"
      shape="circle"
      aria-label="Show task description panel"
      icon={CalendarDays}
      class="mx-auto size-7"
      {...restProps}
    />
  {/snippet}
  <div class="w-40 font-normal normal-case">
    This column shows the current estimate for each task. Estimates are
    expressed in terms of points. Each point is approximately equal to half a
    day.
  </div>
</Tooltip>
