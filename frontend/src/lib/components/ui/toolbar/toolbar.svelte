<script lang="ts">
  import {
    ActionIds,
    getRegistryContext,
  } from "$lib/components/ui/command-palette";
  import { ToolbarButton } from "$lib/components/ui/toolbar";
  import TaskAction from "$lib/dag-table/task-action.svelte";
  import { twMerge } from "tailwind-merge";

  type Props = {
    selected: boolean;
  };
  let { selected }: Props = $props();

  const command = getRegistryContext();

  const base: string[] = [
    ActionIds.DetailPanelClose,
    ActionIds.DetailPanelOpen,
    ActionIds.Undo,
    ActionIds.Redo,
    ActionIds.Search,
  ];

  let actions = $derived(
    base
      .map((id) => command.get(id))
      .filter((action) => action !== undefined)
      .filter((action) => action.enabled()),
  );
</script>

<div
  class={twMerge(
    "standalone-margin flex w-full flex-1 items-center gap-1 overflow-x-scroll border-t p-2",
  )}
>
  {#each actions as action (action.name)}
    <ToolbarButton {...action} />
  {/each}
  {#if selected}
    <TaskAction class="flex flex-1 justify-center p-2" shape="rounded" />
  {/if}
</div>

<style>
  @media not all and (min-width: 640px) {
    @media all and (display-mode: standalone) {
      .standalone-margin {
        padding-bottom: 1.5rem;
      }
    }
  }
</style>
