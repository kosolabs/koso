<script lang="ts">
  import { page } from "$app/stores";
  import { KosoError } from "$lib/api";
  import { type User } from "$lib/auth.svelte";
  import { Alert } from "$lib/components/ui/alert";
  import { Button } from "$lib/components/ui/button";
  import { Editable } from "$lib/components/ui/editable";
  import { DagTable, Koso, KosoSocket } from "$lib/dag-table";
  import { githubInstallUrl } from "$lib/github";
  import { nav } from "$lib/nav.svelte";
  import Navbar from "$lib/navbar.svelte";
  import {
    exportProject,
    fetchProject,
    fetchProjectUsers,
    type Project,
    updateProject,
  } from "$lib/projects";
  import { Action } from "$lib/shortcuts";
  import { FileDown, PlugZap, UserPlus } from "lucide-svelte";
  import { toast } from "svelte-sonner";
  import * as Y from "yjs";
  import ProjectShareModal from "./project-share-modal.svelte";
  import UnauthorizedModal from "./unauthorized-modal.svelte";

  const projectId = $page.params.projectId;
  const koso = new Koso(projectId, new Y.Doc());
  const kosoSocket = new KosoSocket(koso, projectId);
  window.koso = koso;
  window.Y = Y;

  let deflicker: Promise<void> = new Promise((r) => window.setTimeout(r, 50));
  let project: Promise<Project> = loadProject();
  let projectUsersPromise: Promise<User[]> = loadProjectUsers();
  let projectUsers: User[] = $state([]);
  let openShareModal: boolean = $state(false);

  async function loadProjectUsers() {
    const users = await fetchProjectUsers(projectId);

    projectUsers = users;
    return projectUsers;
  }

  async function loadProject() {
    return await fetchProject(projectId);
  }

  async function saveEditedProjectName(name: string) {
    let updatedProject;
    try {
      updatedProject = await updateProject({ projectId, name });
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
    toast.info("Exporting project...");
    const projectExport = await exportProject(projectId);

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

  export const extraActions: Action[] = [
    new Action({
      callback: exportProjectToFile,
      title: "Export Project",
      description: "Export project to JSON",
      icon: FileDown,
      toolbar: false,
    }),
  ];

  $effect(() => {
    if (kosoSocket.online && kosoSocket.authorized) {
      nav.lastVisitedProjectId = $page.params.projectId;
    }
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
    <Button title="Connect to Github" href={githubInstallUrl(projectId)}>
      <PlugZap />
    </Button>
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

{#if !kosoSocket.online}
  <div class="m-4">
    <Alert>Connection to server lost. Working offline.</Alert>
  </div>
{/if}

<UnauthorizedModal open={!kosoSocket.authorized} />

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

  <DagTable {koso} users={projectUsers} {extraActions} />
{/await}
