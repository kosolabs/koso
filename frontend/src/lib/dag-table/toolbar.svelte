<script lang="ts">
  import { ToolbarButton } from "$lib/components/ui/toolbar-button";
  import { cn } from "$lib/kosui/utils";
  import type { Action } from "$lib/shortcuts";
  import type { Snippet } from "svelte";

  type Props = {
    children: Snippet;
    actions: Action[];
  };
  let { children, actions }: Props = $props();

  const toolbarActions = $derived(
    actions.filter((action) => action.toolbar && action.enabled()),
  );

  let toolbarHeight: number = $state(0);
  function height(el: HTMLDivElement) {
    toolbarHeight = el.offsetHeight;
  }
</script>

<div
  use:height
  class={cn(
    "max-sm-standalone-margin fixed bottom-0 left-0 z-10 flex w-full items-center overflow-x-scroll px-2 py-1 backdrop-blur-xs max-sm:border-t sm:sticky sm:top-0 sm:gap-2 sm:border-b",
  )}
>
  {#each toolbarActions as action}
    <ToolbarButton {...action} />
  {/each}
</div>
<div class="toolbar-margin p-2" style="--toolbar-height: {toolbarHeight}px">
  {@render children()}
</div>

<style>
  @media not all and (min-width: 640px) {
    .toolbar-margin {
      margin-bottom: var(--toolbar-height);
    }
  }

  @media not all and (min-width: 640px) {
    @media all and (display-mode: standalone) {
      .max-sm-standalone-margin {
        padding-bottom: 1.5rem;
      }
    }
  }
</style>
