<script lang="ts">
  import { goto } from "$app/navigation";
  import { KosoError } from "$lib/api";
  import { Alert } from "$lib/components/ui/alert";
  import { Button } from "$lib/components/ui/button";
  import Navbar from "$lib/navbar.svelte";
  import {
    fetchProjects,
    createProject as projectsCreateProject,
    deleteProject as projectsDeleteProject,
    type Project,
    type ProjectExport,
  } from "$lib/projects";
  import { HardDriveUpload, Layers, PackagePlus, Trash2 } from "lucide-svelte";
  import { toast } from "svelte-sonner";

  let deflicker: Promise<Project[]> = new Promise((r) => setTimeout(r, 50));
  let projects: Promise<Project[]> = fetchProjects();
  let errorMessage: string | null = null;

  async function createProject(projectExport: ProjectExport | null = null) {
    errorMessage = null;
    let project;
    try {
      project = await projectsCreateProject(projectExport);
    } catch (err) {
      if (err instanceof KosoError && err.hasReason("TOO_MANY_PROJECTS")) {
        errorMessage =
          "Cannot create new project, you already have too many. Contact us for more!";
      } else if (err instanceof KosoError && err.status === 422) {
        errorMessage =
          "The Koso export file is malformed. Verify the correct file was selected and try again.";
      } else {
        errorMessage = "Something went wrong. Please try again.";
        console.warn(err);
      }
      return;
    }
    await goto(`/projects/${project.projectId}`);

    toast.info("Project created!");
  }

  function triggerFileSelect() {
    document.getElementById("projectImportFileInput")?.click();
  }

  async function deleteProject(project: Project) {
    toast.promise(projectsDeleteProject(project), {
      duration: 10000,
      loading: `Moving ${project.name} to the trash...`,
      success: (project) => {
        projects = fetchProjects();
        return `${project.name} has been placed in the trash and will be permanently deleted in 30 days.`;
      },
      error: (err) => {
        if (err instanceof KosoError) {
          return `Could not move ${project.name} to the trash: ${err.message}`;
        }
        console.warn(err);
        return "Something went wrong. Please try again.";
      },
    });
  }

  function parseProjectExport(data: string) {
    try {
      return JSON.parse(data);
    } catch (e) {
      errorMessage =
        "The Koso export file is malformed. Verify the correct file was selected and try again.";
      toast.error(
        "The Koso export file is malformed. Verify the correct file was selected and try again.",
      );
      throw e;
    }
  }

  async function importProject(
    event: Event & {
      currentTarget: EventTarget & HTMLInputElement;
    },
  ) {
    const files = event.currentTarget.files;
    const file = files && files.item(0);
    if (!file) {
      return;
    }

    errorMessage = null;
    if (files.length > 1) {
      errorMessage = "Select a single file.";
      return;
    }

    let projectExport = parseProjectExport(await file.text());
    await createProject(projectExport);
  }
</script>

<Navbar />

{#if errorMessage}
  <div class="m-4 flex-grow-0">
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
  {@const filteredProjects = projects.filter((p) => !p.deletedOn)}
  <input
    id="projectImportFileInput"
    type="file"
    accept=".json,application/JSON"
    multiple={false}
    hidden
    onchange={importProject}
  />

  {#if filteredProjects.length === 0}
    <div
      class="m-2 flex flex-col items-center gap-6 rounded border bg-card p-8"
    >
      <div><Layers /></div>
      <div class="text-xl">Create your first Koso project!</div>
      <div>
        <Button title="New Project" onclick={() => createProject()}>
          <PackagePlus class="me-2 w-5" />New
        </Button>
        <Button title="Import Project" onclick={triggerFileSelect}>
          <HardDriveUpload class="me-2 w-5" /> Import
        </Button>
      </div>
    </div>
  {:else}
    <div class="m-2 flex flex-col rounded border">
      <div class="flex flex-col items-end p-2">
        <div>
          <Button title="New Project" onclick={() => createProject()}>
            <PackagePlus class="me-2 w-5" />New
          </Button>
          <Button title="Import Project" onclick={triggerFileSelect}>
            <HardDriveUpload class="me-2 w-5" />Import
          </Button>
        </div>
      </div>
      <div class="flex flex-col items-stretch [&>*:nth-child(even)]:bg-muted">
        {#each filteredProjects as project}
          <div class="flex items-center border-t p-2">
            <Button
              variant="link"
              class="text-lg"
              href="projects/{project.projectId}"
            >
              {project.name}
            </Button>
            <Button
              title="Move Project to Trash"
              class="ml-auto"
              variant="outline"
              aria-label="Delete {project.name}"
              onclick={() => deleteProject(project)}
            >
              <Trash2 class="me-2 w-5" />Trash
            </Button>
          </div>
        {/each}
      </div>
    </div>
  {/if}
{/await}
