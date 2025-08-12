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
      taskId: "T1",
      title: "Fix login bug",
      description: "Resolve 500 error when logging in",
      parentTask: "Auth Module",
    },
    {
      taskId: "T2",
      title: "Login error fix",
      description: "Investigate and fix login issues",
      parentTask: "Authentication",
    },
  ];
</script>

<!-- Navbar -->
<div class="flex h-dvh flex-col">
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

<div>Hello</div>
<!-- Card Layout -->
<Card
  taskID="1234"
  title="Task 1"
  description="Description of Task 1"
  parentTask="Parent Task Name"
/>
