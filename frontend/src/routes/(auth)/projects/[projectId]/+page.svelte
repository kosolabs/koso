<script lang="ts">
  import { page } from "$app/stores";
  import { token, user, type User } from "$lib/auth";
  import { Alert } from "$lib/components/ui/alert";
  import { Button } from "$lib/components/ui/button";
  import { Editable } from "$lib/components/ui/editable";
  import { DagTable } from "$lib/dag-table";
  import { Koso } from "$lib/koso";
  import { lastVisitedProjectId } from "$lib/nav";
  import Navbar from "$lib/navbar.svelte";
  import {
    fetchProject,
    fetchProjectUsers,
    type Project,
    updateProject,
  } from "$lib/projects";
  import { KosoSocket } from "$lib/socket";
  import { UserPlus } from "lucide-svelte";
  import { onDestroy, onMount } from "svelte";
  import * as Y from "yjs";
  import ProjectShareModal from "./project-share-modal.svelte";
  import UnauthorizedModal from "./unauthorized-modal.svelte";

  const projectId = $page.params.projectId;
  const koso = new Koso(projectId, new Y.Doc());
  window.koso = koso;
  window.Y = Y;

  let projectPromise: Promise<Project> = loadProject();
  let project: Project | null = null;
  let projectUsersPromise: Promise<User[]> = loadProjectUsers();
  let projectUsers: User[] = [];
  let openShareModal = false;

  async function loadProjectUsers() {
    if (!$user || !$token) throw new Error("User is unauthorized");
    let users = await fetchProjectUsers($token, projectId);

    projectUsers = users;
    return projectUsers;
  }

  async function loadProject() {
    if (!$user || !$token) throw new Error("User is unauthorized");

    project = await fetchProject($token, projectId);
    return project;
  }

  async function saveEditedProjectName(name: string) {
    if (!$user || !$token) throw new Error("User is unauthorized");
    if (!project) {
      throw new Error("Project not loaded yet. Maybe loadProject failed?");
    }

    const updatedProject = await updateProject($token, {
      project_id: projectId,
      name,
    });

    project.name = updatedProject.name;
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

    await kosoSocket.openWebSocket();
    $lastVisitedProjectId = $page.params.projectId;
  });

  onDestroy(() => {
    kosoSocket.closeAndShutdown(1000, "Closed in onDestroy.");
  });
</script>

<Navbar>
  <svelte:fragment slot="left-items">
    <div>
      {#await projectPromise then project}
        <Editable
          class="ml-2 text-lg"
          value={project.name}
          aria-label="Set project name"
          onsave={saveEditedProjectName}
          onkeydown={(e) => e.stopPropagation()}
        />
      {/await}
    </div>
  </svelte:fragment>
  <svelte:fragment slot="right-items">
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

<UnauthorizedModal bind:open={showUnauthorizedModal} />
{#await projectPromise then project}
  {#await projectUsersPromise then _}
    <ProjectShareModal bind:open={openShareModal} bind:projectUsers {project} />
  {/await}
{/await}

<DagTable {koso} users={projectUsers} />
