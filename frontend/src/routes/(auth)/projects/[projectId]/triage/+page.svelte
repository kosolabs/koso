<script lang="ts">
  import { getAuthContext } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { getProjectContext } from "$lib/dag-table";
  import Card from "$lib/components/ui/card/Card.svelte";

  const auth = getAuthContext();
  const project = getProjectContext();

  // Mock Data
  let pair = [
    {
      taskID: "T1",
      taskName: "Fix login bug",
      taskDescription: "Resolve 500 error when logging in",
      parentTask: "Auth Module",
    },
    {
      taskID: "T2",
      taskName: "Login error fix",
      taskDescription: "Investigate and fix login issues",
      parentTask: "Authentication",
    },
  ];
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
<div class="flex min-h-screen items-center justify-center gap-8">
  {#each pair as task}
    <Card
      taskID={task.taskID}
      taskName={task.taskName}
      taskDescription={task.taskDescription}
      parentTask={task.parentTask}
    />
  {/each}
</div>
