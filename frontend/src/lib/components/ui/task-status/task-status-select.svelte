<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { Portal } from "$lib/components/ui/portal";
  import { ResponsiveText } from "$lib/components/ui/responsive-text";
  import type { Status } from "$lib/koso";
  import { DropdownMenuMonitoredRoot } from "$lib/popover-monitors";
  import Confetti from "svelte-confetti";
  import { TaskStatusIcon } from ".";

  const statuses: Status[] = ["Not Started", "In Progress", "Done"];

  type Props = {
    value: Status | null;
    statusTime: Date | null;
    closeFocus: HTMLElement;
    onselect: (status: Status) => void;
  };
  let {
    value = $bindable(),
    statusTime,
    closeFocus,
    onselect,
  }: Props = $props();
  let showConfetti: boolean = $state(false);

  function select(status: Status) {
    value = status;
    showConfetti = status === "Done";
    onselect(status);
  }
</script>

<DropdownMenuMonitoredRoot {closeFocus}>
  <DropdownMenu.Trigger
    class="flex items-center gap-2"
    title={(value || "Not Started") +
      (statusTime ? " - " + statusTime.toLocaleString() : "")}
  >
    <TaskStatusIcon status={value} />
    <ResponsiveText>{value || "Not Started"}</ResponsiveText>
  </DropdownMenu.Trigger>
  <DropdownMenu.Content>
    {#each statuses as status}
      <DropdownMenu.Item
        class="flex items-center gap-2 rounded p-2"
        on:click={() => select(status)}
      >
        <TaskStatusIcon {status} />
        {status}
      </DropdownMenu.Item>
    {/each}
  </DropdownMenu.Content>
</DropdownMenuMonitoredRoot>

{#if showConfetti}
  <Portal>
    <div class="fixed left-1/2 top-1/2">
      <Confetti />
    </div>
  </Portal>
{/if}
