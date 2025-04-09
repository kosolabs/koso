<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { showUnauthorizedDialog, type User } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { Koso, KosoSocket, newInboxContext, TaskTable } from "$lib/dag-table";
  import OfflineAlert from "$lib/dag-table/offline-alert.svelte";
  import { Button } from "$lib/kosui/button";
  import { nav } from "$lib/nav.svelte";
  import { fetchProject, fetchProjectUsers, type Project } from "$lib/projects";
  import { Notebook } from "lucide-svelte";
  import * as Y from "yjs";

  const projectId = page.params.projectId;
  nav.lastVisitedProjectId = projectId;

  const koso = new Koso(projectId, new Y.Doc());
  const kosoSocket = new KosoSocket(koso, projectId);
  window.koso = koso;
  window.Y = Y;

  newInboxContext();

  let deflicker: Promise<void> = new Promise((r) => window.setTimeout(r, 50));
  let project: Promise<Project> = fetchProject(projectId);
  let projectUsersPromise: Promise<User[]> = loadProjectUsers();
  let projectUsers: User[] = $state([]);

  async function loadProjectUsers() {
    const users = await fetchProjectUsers(projectId);

    projectUsers = users;
    return projectUsers;
  }

  $effect(() => {
    if (kosoSocket.unauthorized) {
      showUnauthorizedDialog();
    }
  });
</script>

<Navbar>
  {#snippet left()}
    <div>
      <h1 class="ml-2 text-lg">
        {#await project}
          Inbox
        {:then project}
          Inbox - {project.name}
        {/await}
      </h1>
    </div>
  {/snippet}
  {#snippet right()}
    <Button
      variant="plain"
      shape="circle"
      tooltip="Project planning view"
      aria-label="Project planning view"
      onclick={() => goto(`/projects/${projectId}`)}
      class="p-2"
    >
      <Notebook size={20} />
    </Button>
  {/snippet}
</Navbar>

<OfflineAlert offline={kosoSocket.offline} />

{#await projectUsersPromise}
  {#await deflicker}
    <!-- Deflicker load. -->
  {:then}
    <!-- TODO: Make this a Skeleton -->
    <div class="flex flex-col items-center justify-center rounded border p-4">
      <div class="text-l">Loading...</div>
    </div>
  {/await}
{:then}
  <TaskTable {koso} users={projectUsers} />
{/await}
