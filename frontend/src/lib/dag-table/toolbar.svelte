<script lang="ts">
  import { command, type ActionID } from "$lib/components/ui/command-palette";
  import { ToolbarButton } from "$lib/components/ui/toolbar-button";
  import { twMerge } from "tailwind-merge";

  type Props = {
    actions: ActionID[];
  };
  let props: Props = $props();
  let actions = $derived(
    props.actions
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
  {#each actions as action (action.title)}
    <ToolbarButton {...action} />
  {/each}
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
