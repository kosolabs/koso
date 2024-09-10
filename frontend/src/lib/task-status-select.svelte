<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { createEventDispatcher } from "svelte";
  import Confetti from "svelte-confetti";
  import type { Status } from "./koso";
  import ResponsiveText from "./responsive-text.svelte";
  import TaskStatusIcon from "./task-status-icon.svelte";

  const dispatch = createEventDispatcher<{ select: Status }>();
  const statuses: Status[] = ["Not Started", "In Progress", "Done"];

  export let value: Status | null;

  let showConfetti: boolean = false;

  function select(status: Status) {
    value = status;
    showConfetti = status === "Done";
    dispatch("select", status);
  }
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger class="flex items-center gap-2">
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
        <ResponsiveText>{status}</ResponsiveText>
      </DropdownMenu.Item>
    {/each}
  </DropdownMenu.Content>
</DropdownMenu.Root>

{#if showConfetti}
  <Confetti />
{/if}
