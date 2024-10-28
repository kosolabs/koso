<script lang="ts">
  import { page } from "$app/stores";
  import { token, user, type User } from "$lib/auth";
  import { Alert } from "$lib/components/ui/alert";
  import { Button } from "$lib/components/ui/button";
  import { Editable } from "$lib/components/ui/editable";
  import { DagTable } from "$lib/dag-table";
  import { Koso } from "$lib/koso.svelte";
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
  import { toast } from "svelte-sonner";
  import * as Y from "yjs";
  import ProjectShareModal from "./project-share-modal.svelte";
  import UnauthorizedModal from "./unauthorized-modal.svelte";
  import { KosoError } from "$lib/api";

  const projectId = $page.params.projectId;
  const koso = new Koso(projectId, new Y.Doc());
  window.koso = koso;
  window.Y = Y;

  let deflicker: Promise<Project[]> = new Promise((r) => setTimeout(r, 50));
  let project: Promise<Project> = loadProject();
  let projectUsersPromise: Promise<User[]> = loadProjectUsers();
  let projectUsers: User[] = [];
  let openShareModal = false;

  async function loadProjectUsers() {
    if (!$user || !$token) throw new Error("User is unauthorized");
    const users = await fetchProjectUsers($token, projectId);

    projectUsers = users;
    return projectUsers;
  }

  async function loadProject() {
    if (!$user || !$token) throw new Error("User is unauthorized");

    return await fetchProject($token, projectId);
  }

  async function saveEditedProjectName(name: string) {
    if (!$user || !$token) throw new Error("User is unauthorized");

    let updatedProject;
    try {
      updatedProject = await updateProject($token, {
        project_id: projectId,
        name,
      });
    } catch (err) {
      if (err instanceof KosoError && err.hasReason("EMPTY_NAME")) {
        toast.warning("Project name may not be blank.");
      } else if (err instanceof KosoError && err.hasReason("LONG_NAME")) {
        toast.warning("Project name is too long. Try a shorter one.");
      } else {
        toast.error("Failed to change project name.");
      }
      throw err;
    }
    let p = await project;
    p.name = updatedProject.name;
  }

  async function exportProjectToFile() {
    if (!$user || !$token) throw new Error("User is unauthorized");

    toast.info("Exporting project...");
    const projectExport = await exportProject($token, projectId);

    let p = await project;
    let projectName = (p.name || "project")
      .toLowerCase()
      .trim()
      .replaceAll(/[\s+]/g, "-")
      .replaceAll(/[^-_a-z0-9]/g, "");
    let now = new Date();
    const fileName = `${projectName}-export-${now.getFullYear()}-${now.getMonth()}-${now.getDate()}-${now.getHours()}-${now.getMinutes()}.json`;
    saveJsonFile(JSON.stringify(projectExport, null, 2), fileName);
  }

  function saveJsonFile(json: string, name: string) {
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = name;
    a.click();
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
    await kosoSocket.openWebSocket();
    $lastVisitedProjectId = $page.params.projectId;
  });

  onDestroy(() => {
    kosoSocket.closeAndShutdown(1000, "Closed in onDestroy.");
    koso.destroy();
  });
</script>

<Navbar>
  {#snippet left()}
    <div>
      {#await project then project}
        <Editable
          class="ml-2 text-lg"
          value={project.name}
          aria-label="Set project name"
          onsave={async (name) => {
            await saveEditedProjectName(name);
          }}
          onkeydown={(e) => e.stopPropagation()}
        />
      {/await}
    </div>
  {/snippet}
  {#snippet right()}
    <Button title="Export Project" onclick={exportProjectToFile}>
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
  {/snippet}
</Navbar>

{#if showSocketOfflineAlert}
  <div class="m-4">
    <Alert>Connection to server lost. Working offline.</Alert>
  </div>
{/if}

<UnauthorizedModal bind:open={showUnauthorizedModal} />

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
  {#await project then project}
    <ProjectShareModal bind:open={openShareModal} bind:projectUsers {project} />
  {/await}

  <DagTable {koso} users={projectUsers} />
{/await}
