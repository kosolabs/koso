<script lang="ts">
  import { page } from "$app/stores";
  import { token, user, type User } from "$lib/auth";
  import { Alert } from "$lib/components/ui/alert";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Editable } from "$lib/components/ui/editable";
  import { DagTable } from "$lib/dag-table";
  import { Koso } from "$lib/koso";
  import { lastVisitedProjectId } from "$lib/nav";
  import Navbar from "$lib/navbar.svelte";
  import {
    exportProject,
    fetchProject,
    fetchProjectUsers,
    type Project,
    updateProject,
  } from "$lib/projects";
  import { KosoSocket } from "$lib/socket";
  import { FileDown, UserPlus } from "lucide-svelte";
  import { onDestroy, onMount } from "svelte";
  import * as Y from "yjs";
  import ProjectShareModal from "./project-share-modal.svelte";
  import UnauthorizedModal from "./unauthorized-modal.svelte";

  const projectId = $page.params.projectId;
  const koso = new Koso(projectId, new Y.Doc());
  window.koso = koso;
  window.Y = Y;

  let project: Project | null = null;
  let projectUsers: User[] = [];
  let openShareModal = false;

  async function loadProjectUsers() {
    if (!$user || !$token) throw new Error("User is unauthorized");
    return await fetchProjectUsers($token, projectId);
  }

  async function loadProject() {
    if (!$user || !$token) throw new Error("User is unauthorized");

    return await fetchProject($token, projectId);
  }

  async function saveEditedProjectName(name: string) {
    if (!$user || !$token) throw new Error("User is unauthorized");

    const updatedProject = await updateProject($token, {
      project_id: projectId,
      name,
    });

    if (project) {
      project.name = updatedProject.name;
    }
  }

  let showProjectExport: boolean = false;
  let downloadProjectExport: string | null = null;
  async function doExportProject() {
    if (!$user || !$token) throw new Error("User is unauthorized");
    showProjectExport = true;

    let projectExport = await exportProject($token, projectId);
    downloadProjectExport = JSON.stringify(projectExport);
    navigator.clipboard.writeText(downloadProjectExport);
  }

  let showSocketOfflineAlert: boolean = false;
  let showUnauthorizedModal: boolean = false;

  const kosoSocket = new KosoSocket(
    koso,
    projectId,
    () => $token,
    () => {
      showUnauthorizedModal = true;
    },
    () => {
      showSocketOfflineAlert = false;
    },
    () => {
      showSocketOfflineAlert = true;
    },
  );

  onMount(async () => {
    if (!$user || !$token) {
      return;
    }

    [projectUsers, project] = await Promise.all([
      loadProjectUsers(),
      loadProject(),
      kosoSocket.openWebSocket(),
    ]);
    $lastVisitedProjectId = $page.params.projectId;
  });

  onDestroy(() => {
    kosoSocket.closeAndShutdown(1000, "Closed in onDestroy.");
  });
</script>

<Navbar>
  <svelte:fragment slot="left-items">
    <div>
      {#if project}
        <Editable
          class="ml-2 text-lg"
          value={project.name}
          aria-label="Set project name"
          onsave={saveEditedProjectName}
          onkeydown={(e) => e.stopPropagation()}
        />
      {/if}
    </div>
  </svelte:fragment>
  <svelte:fragment slot="right-items">
    <Button title="Export Project" onclick={doExportProject}>
      <FileDown />
    </Button>
    <Button
      title="Share Project"
      onclick={() => {
        openShareModal = true;
      }}
    >
      <UserPlus />
    </Button>
  </svelte:fragment>
</Navbar>

{#if showSocketOfflineAlert}
  <div class="m-4">
    <Alert>Connection to server lost. Working offline.</Alert>
  </div>
{/if}

<Dialog.Root bind:open={showProjectExport}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Export</Dialog.Title>
    </Dialog.Header>

    {#if downloadProjectExport}
      <div>Copied to Clipboard</div>
      <div>
        <textarea readonly class="h-48 w-full">{downloadProjectExport}</textarea
        >
      </div>
    {:else}
      <div>Loading...</div>
    {/if}
  </Dialog.Content>
</Dialog.Root>

<UnauthorizedModal bind:open={showUnauthorizedModal} />
<ProjectShareModal bind:open={openShareModal} bind:projectUsers {project} />

<DagTable {koso} users={projectUsers} />
