<script lang="ts">
  import { goto } from "$app/navigation";
  import { showUnauthorizedDialog } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { TaskTable } from "$lib/dag-table";
  import { newInboxContext } from "$lib/dag-table/inbox-context.svelte";
  import OfflineAlert from "$lib/dag-table/offline-alert.svelte";
  import { getProjectContext } from "$lib/dag-table/project-context.svelte";
  import { Button } from "$lib/kosui/button";
  import { Notebook } from "lucide-svelte";

  const project = getProjectContext();
  const { koso } = project;
  newInboxContext(koso);

  $effect(() => {
    if (project.socket.unauthorized) {
      showUnauthorizedDialog();
    }
  });
</script>

<Navbar>
  {#snippet left()}
    <div>
      <h1 class="ml-2 text-lg">
        Inbox - {project.name}
      </h1>
    </div>
  {/snippet}
  {#snippet right()}
    <Button
      variant="plain"
      shape="circle"
      tooltip="Project planning view"
      aria-label="Project planning view"
      onclick={() => goto(`/projects/${project.id}`)}
      class="p-2"
    >
      <Notebook size={20} />
    </Button>
  {/snippet}
</Navbar>

<OfflineAlert offline={project.socket.offline} />

<TaskTable users={project.users} />
