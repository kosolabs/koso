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
  let currentDupeIndex: number = $state(0);

  let currentDupe: Dupe | null = $derived.by(() => {
    if (currentDupeIndex >= dupes.length) {
      return null;
    }
    for (let i = currentDupeIndex; i < dupes.length; i++) {
      const dupe = dupes[i];
      if (
        project.koso.graph.has(dupe.task1Id) &&
        project.koso.graph.has(dupe.task2Id)
      ) {
        return dupe;
      }
    }
    return null;
  });

  onMount(async () => {
    try {
      dupes = await fetchDupes(auth, project.id);
      // TODO: Filter dupes to show only unresolved
      console.log("[triage Duplicates fetched:", dupes);
    } catch (err) {
      console.error("Error fetching duplicates:", err);
    }
  });

  $effect(() => {
    console.log("[triage] Current dupe pair:", dupes);
  });

  function handleYesOnClick() {
    currentDupeIndex++;
    // TODO: Send a patch call to confirm the dupe and merge the tasks
  }

  function handleNoOnClick() {
    currentDupeIndex++;
    // TODO: Send a patch call to decline the dupe
  }

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

<!-- {#if noRealTask} -->
<!-- <div> No dupes / fake task </div> -->
<!-- {/if} -->

{#if currentDupe}
  <div class="grid grid-cols-1 gap-4 p-10">
    <Card
      dupeId={currentDupe.dupeId}
      taskId={currentDupe.task1Id}
      similarity={currentDupe.similarity}
    />
    <Card
      dupeId={currentDupe.dupeId}
      taskId={currentDupe.task2Id}
      similarity={currentDupe.similarity}
    />
  </div>

  <div class="flex items-center justify-center gap-8 p-4 font-bold text-white">
    <Button onclick={handleYesOnClick} variant="filled">Yes / Merge</Button>
    <Button onclick={handleNoOnClick} variant="filled"
      >No / Keep Separate</Button
    >
  </div>
{:else if currentDupeIndex > 0}
  <p>Great Job! All duplicates resolved!</p>
{:else}
  <p class="p-10 text-gray-400">No duplicates found 🎉</p>
{/if}
