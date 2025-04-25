<script lang="ts">
  import { Navbar } from "$lib/components/ui/navbar";
  import { TaskTable } from "$lib/dag-table";
  import { newInboxContext } from "$lib/dag-table/inbox-context.svelte";
  import OfflineAlert from "$lib/dag-table/offline-alert.svelte";
  import { getProjectContext } from "$lib/dag-table/project-context.svelte";

  const project = getProjectContext();
  const { koso } = project;
  newInboxContext(koso);
</script>

<Navbar>
  {#snippet left()}
    <div>
      <h1 class="ml-2 text-lg">
        Inbox - {project.name}
      </h1>
    </div>
  {/snippet}
</Navbar>

<OfflineAlert offline={project.socket.offline} />

<TaskTable users={project.users} />
