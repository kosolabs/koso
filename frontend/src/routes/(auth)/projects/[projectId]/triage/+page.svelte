<script lang="ts">
  import { getAuthContext } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { getProjectContext } from "$lib/dag-table";
  import { onMount } from "svelte";
  import Card from "$lib/components/ui/card/Card.svelte";
  import { Button } from "kosui";
  import { fetchDupes } from "$lib/projects";
  import type { DupePair } from "$lib/projects";

  const auth = getAuthContext();
  const project = getProjectContext();

  let pair: DupePair[] = [];

  onMount(async () => {
    try {
      pair = await fetchDupes(auth, project.id);
      console.log("[triage Duplicates fetched:", pair);
    } catch (err) {
      console.error("Error fetching duplicates:", err);
    }
  });
</script>

<!-- Navbar -->
<div class="flex-col">
  <div class="grow-0">
    <Navbar breadcrumbs={["Projects", project.name, "Triage"]}>
      {#snippet left()}
        <div>
          <h1 class="ml-2 text-lg">Triage</h1>
        </div>
      {/snippet}
    </Navbar>
  </div>
</div>

<!-- Card Layout -->
<div class="items-center justify-center gap-8">
  <div class="flex items-center justify-center p-10">
    {#each pair as task}
      <Card
        taskID={task.taskID}
        taskName={task.taskName}
        taskDescription={task.taskDescription}
        parentTask={task.parentTask}
      />
    {/each}
  </div>
  <div class="flex items-center justify-center gap-8 p-4 font-bold text-white">
    <Button variant="filled">Yes / Merge</Button>
    <Button variant="filled">No / Keep Separate</Button>
  </div>
</div>
