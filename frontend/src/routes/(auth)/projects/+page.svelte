<script lang="ts">
  import { goto } from "$app/navigation";
  import { KosoError } from "$lib/api";
  import { Alert } from "$lib/components/ui/alert";
  import { Button } from "$lib/components/ui/button";
  import Navbar from "$lib/navbar.svelte";
  import {
    fetchProjects,
    createProject as projectsCreateProject,
    type Project,
  } from "$lib/projects";
  import { Layers } from "lucide-svelte";
  import ImportProject from "./import-project.svelte";

  let deflicker: Promise<Project[]> = new Promise((r) => setTimeout(r, 50));
  let projects: Promise<Project[]> = fetchProjects();
  let errorMessage: string | null = null;
  let importProjectDialogOpen: boolean = false;

  async function createProject(import_data: string | null = null) {
    errorMessage = null;
    let project;
    try {
      project = await projectsCreateProject(import_data);
    } catch (err) {
      if (err instanceof KosoError && err.hasReason("TOO_MANY_PROJECTS")) {
        errorMessage =
          "Cannot create new project, you already have too many. Contact us for more!";
      } else if (
        err instanceof KosoError &&
        err.hasReason("MALFORMED_IMPORT")
      ) {
        errorMessage =
          "The selected import file is malformed. Verify the correct file was selected and try again.";
      } else {
        errorMessage = "Something went wrong. Please try again.";
      }
      return;
    }
    await goto(`/projects/${project.project_id}`);
  }

  function showImportProjectDialog() {
    importProjectDialogOpen = true;
  }
</script>

<Navbar />

{#if errorMessage}
  <div class="my-2 flex-grow-0">
    <Alert variant="destructive">{errorMessage}</Alert>
  </div>
{/if}

{#await projects}
  {#await deflicker}
    <!-- Deflicker load. -->
  {:then}
    <!-- TODO: Make this a Skeleton -->
    <div class="flex flex-col items-center justify-center rounded border p-4">
      <div class="text-l">Loading...</div>
    </div>
  {/await}
{:then projects}
  {#if projects.length === 0}
    <div
      class="m-4 flex flex-col items-center gap-6 rounded border bg-card p-8"
    >
      <div><Layers /></div>
      <div class="text-xl">Create your first Koso project!</div>
      <div>
        <Button onclick={() => createProject()}>New project</Button>
        <Button onclick={() => showImportProjectDialog()}>
          Import project
        </Button>
      </div>
    </div>
  {:else}
    <div class="m-4 flex flex-col rounded border">
      <div class="flex flex-col items-end p-2">
        <div>
          <Button onclick={() => createProject()}>New project</Button>
          <Button onclick={() => showImportProjectDialog()}>
            Import project
          </Button>
        </div>
      </div>
      <div class="flex flex-col items-stretch [&>*:nth-child(even)]:bg-muted">
        {#each projects as project}
          <div class="border-t p-2">
            <Button
              variant="link"
              class="text-lg"
              href="projects/{project.project_id}"
            >
              {project.name}
            </Button>
          </div>
        {/each}
      </div>
    </div>
  {/if}
{/await}

<ImportProject bind:open={importProjectDialogOpen} />
