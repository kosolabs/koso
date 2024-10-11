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
    "z-10 flex items-center overflow-x-scroll px-2 backdrop-blur-sm",
    "fixed bottom-0 left-0 h-12 w-full border-t",
    "sm:sticky sm:top-0 sm:gap-2 sm:border-b",
  )}
>
  {#each toolbarActions as action}
    <ToolbarButton {...action} />
  {/each}
</div>
