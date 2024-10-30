<script lang="ts">
  import { ToolbarButton } from "$lib/components/ui/toolbar-button";
  import type { Action } from "$lib/shortcuts";
  import { cn } from "$lib/utils";

  type Props = {
    actions: Action[];
  };
  let { actions }: Props = $props();

  const toolbarActions = $derived(
    actions.filter((action) => action.toolbar && action.enabled()),
  );
</script>

<div
  class={cn(
    "sticky left-0 top-0 z-10 flex w-full items-center overflow-x-scroll border-b p-2 backdrop-blur-sm sm:gap-2",
  )}
>
  {#each toolbarActions as action}
    <ToolbarButton {...action} />
  {/each}
</div>
