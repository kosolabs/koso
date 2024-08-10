<script lang="ts">
  import { Dropdown } from "flowbite-svelte";
  import { createEventDispatcher } from "svelte";
  import Confetti from "svelte-confetti";
  import type { Status } from "./koso";
  import TaskStatusIcon from "./task-status-icon.svelte";

  const dispatch = createEventDispatcher<{ select: Status }>();
  const statuses: Status[] = ["Not Started", "In Progress", "Done"];

  export let value: Status | null;

  $: console.log(value);

  let open: boolean = false;
  let showConfetti: boolean = false;

  function select(status: Status) {
    value = status;
    open = false;
    showConfetti = status === "Done";
    dispatch("select", status);
  }
</script>

<button class="flex items-center gap-1">
  <TaskStatusIcon status={value} />
  <div class="whitespace-nowrap text-sm max-md:hidden">
    {value || "Not Started"}
  </div>
</button>
<Dropdown bind:open>
  <div class="flex flex-col gap-2 p-2">
    {#each statuses as status}
      <button on:click={() => select(status)}>
        <div
          class="flex items-center gap-2 rounded p-2 text-left hover:bg-primary-100"
        >
          <TaskStatusIcon {status} />
          <div class="whitespace-nowrap text-sm">{status}</div>
        </div>
      </button>
    {/each}
  </div>
</Dropdown>
{#if showConfetti}
  <Confetti />
{/if}
