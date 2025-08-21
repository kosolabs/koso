<script lang="ts">
  import { getAuthContext } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { getProjectContext } from "$lib/dag-table";
  import { onMount } from "svelte";
  import Card from "$lib/components/ui/card/Card.svelte";
  import { Button } from "kosui";
  import { fetchDupes } from "$lib/projects";
  import type { Dupe } from "$lib/projects";
  // import type {ProjectExport} from "$lib/projects";

  const auth = getAuthContext();
  const project = getProjectContext();

  let dupes: Dupe[] = $state([]);
  let onlyDupe: Dupe = $state({} as Dupe);

  onMount(async () => {
    try {
      dupes = await fetchDupes(auth, project.id);
      if (dupes.length == 1) {
        onlyDupe = dupes[0];
      }

      console.log("[triage Duplicates fetched:", dupes);
    } catch (err) {
      console.error("Error fetching duplicates:", err);
    }
  });

  $effect(() => {
    console.log("[triage] Current dupe pair:", dupes);
  });

  // validation
  // project.koso.graph.has(dupes.task1Id);

  // retrieving task
  // project.koso.getTask(dupes.task1Id).desc;

  // Mock Data
  // let pair: DupePair[] = [
  //   {
  //     taskID: "T1",
  //     taskName: "Fix login bug",
  //     taskDescription: "Resolve 500 error when logging in",
  //     parentTask: "Auth Module",
  //   },
  //   {
  //     taskID: "T2",
  //     taskName: "Login error fix",
  //     taskDescription: "Investigate and fix login issues",
  //     parentTask: "Authentication",
  //   },
  // ];

  // function nextPair() {
  //   if (currentIndex < dupes.length - 1) currentIndex++;
  // }
  // function prevPair() {
  //   if (currentIndex > 0) currentIndex--;
  // }
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
<!-- <div class="items-center gap-8">
  <div class="grid grid-cols-1 gap-4 p-10">
    <Card
      dupeId={dupes.dupeId}
      task1Id={dupes.task1Id}
      similarity={dupes.similarity}
    />
    <Card task2Id={dupes.task2Id} />
  </div>
  <div class="flex items-center justify-center gap-8 p-4 font-bold text-white">
    <Button variant="filled">Yes / Merge</Button>
    <Button variant="filled">No / Keep Separate</Button>
  </div>
</div> -->

{#if onlyDupe}
  <div class="grid grid-cols-1 gap-4 p-10">
    <Card
      dupeId={onlyDupe.dupeId}
      task1Id={onlyDupe.task1Id}
      similarity={onlyDupe.similarity}
    />
    <Card task2Id={onlyDupe.task2Id} />
  </div>

  <div class="flex items-center justify-center gap-8 p-4 font-bold text-white">
    <Button variant="filled">Yes / Merge</Button>
    <Button variant="filled">No / Keep Separate</Button>
  </div>
{:else}
  <p class="p-10 text-gray-400">No duplicates found 🎉</p>
{/if}
