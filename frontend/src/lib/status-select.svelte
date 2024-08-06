<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { Circle, CircleCheck, CircleFadingArrowUp } from "lucide-svelte";

  const dispatch = createEventDispatcher<{ select: string | null }>();

  const statuses: string[] = ["Not Started", "In Progress", "Done"];

  function select(status: string | null) {
    dispatch("select", status);
  }
</script>

<div class="flex flex-col gap-2 p-2">
  {#each statuses as status}
    <button on:click={() => select(status)}>
      <div
        class="flex items-center gap-2 rounded p-2 text-left hover:bg-primary-100"
      >
        {#if status === "Not Started"}
          <Circle />
        {:else if status == "In Progress"}
          <CircleFadingArrowUp />
        {:else if status == "Done"}
          <CircleCheck />
        {/if}
        <div class="text-sm">{status}</div>
      </div>
    </button>
  {/each}
</div>
