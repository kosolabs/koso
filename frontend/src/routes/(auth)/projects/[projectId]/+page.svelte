<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { token, user, type User } from "$lib/auth";
  import { Button } from "$lib/button";
  import { Alert } from "$lib/components/ui/alert";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { DagTable } from "$lib/dag-table";
  import { Koso } from "$lib/koso";
  import { lastVisitedProjectId } from "$lib/nav";
  import Navbar from "$lib/navbar.svelte";
  import {
    fetchProjects,
    fetchProjectUsers,
    type Project,
    updateProject,
  } from "$lib/projects";
  import { KosoSocket } from "$lib/socket";
  import { UserPlus } from "lucide-svelte";
  import { onDestroy, onMount } from "svelte";
  import * as Y from "yjs";
  import ProjectShareModal from "./project-share-modal.svelte";

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

    const projects = await fetchProjects($token);
    for (const project of projects) {
      if (project.project_id == projectId) {
        return project;
      }
    }
    throw new Error(
      `Project ${projectId} does not exist or user no longer has access: ${projects})`,
    );
  }

  let editedProjectName: string | null = null;

  function handleStartEditingProjectName(event: MouseEvent | KeyboardEvent) {
    event.stopPropagation();
    editedProjectName = project?.name || "";
  }

  async function saveEditedProjectName() {
    if (!editedProjectName) {
      editedProjectName = null;
      return;
    }
    if (!$user || !$token) throw new Error("User is unauthorized");

    const updatedProject = await updateProject($token, {
      project_id: projectId,
      name: editedProjectName,
    });

    if (project) {
      project.name = updatedProject.name;
    }
    editedProjectName = null;
  }

  function revertEditedProjectName() {
    if (editedProjectName === null) {
      return;
    }
    editedProjectName = null;
  }

  async function handleEditedProjectNameBlur() {
    await saveEditedProjectName();
  }

  async function handleEditedProjectNameKeydown(event: KeyboardEvent) {
    event.stopPropagation();

    if (event.key === "Escape") {
      revertEditedProjectName();
      event.preventDefault();
      return;
    }

    if (event.key === "Enter") {
      await saveEditedProjectName();
      event.preventDefault();
      event.stopPropagation();
      return;
    }
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
      {#if editedProjectName !== null}
        <Input
          class="ml-2 p-2"
          on:click={(event) => event.stopPropagation()}
          on:blur={handleEditedProjectNameBlur}
          on:keydown={handleEditedProjectNameKeydown}
          bind:value={editedProjectName}
          autofocus
        />
      {:else if project}
        <Button
          data-testid="set-project-name-button"
          variant="link"
          class="text-lg"
          on:click={handleStartEditingProjectName}
          on:keydown={handleStartEditingProjectName}
        >
          {project.name}
        </Button>
      {/if}
    </div>
  </svelte:fragment>
  <svelte:fragment slot="right-items">
    <Button
      title="Share Project"
      on:click={() => {
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

<Dialog.Root bind:open={showUnauthorizedModal}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Unauthorized</Dialog.Title>
      <Dialog.Description>
        You do not have access to the project or the project does not exist.
      </Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer>
      <Button
        on:click={async () => {
          // Don't redirect the user back to a project they don't have access too.
          $lastVisitedProjectId = null;
          await goto("/projects");
        }}>Take me home</Button
      >
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<ProjectShareModal bind:open={openShareModal} bind:projectUsers {project} />

<DagTable {koso} users={projectUsers} />
