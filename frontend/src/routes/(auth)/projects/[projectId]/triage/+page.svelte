<script lang="ts">
  import { getAuthContext } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { getProjectContext } from "$lib/dag-table";
  import Card from "$lib/components/ui/card/Card.svelte";

  const auth = getAuthContext();
  const project = getProjectContext();
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

<!-- Card Layout -->
<Card
  taskID="1234"
  title="Task 1"
  description="Description of Task 1"
  parentTask="Parent Task Name"
/>
