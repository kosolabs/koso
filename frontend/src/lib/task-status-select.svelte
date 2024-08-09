<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { Status } from "./koso";
  import TaskStatus from "./task-status.svelte";

  const dispatch = createEventDispatcher<{ select: Status }>();

  const statuses: Status[] = ["Not Started", "In Progress", "Done"];

  function select(status: Status) {
    dispatch("select", status);
  }
</script>

<div class="flex flex-col gap-2 p-2">
  {#each statuses as status}
    <button on:click={() => select(status)}>
      <div
        class="flex items-center gap-2 rounded p-2 text-left hover:bg-primary-100"
      >
        <TaskStatus {status} />
      </div>
    </button>
  {/each}
</div>
