<script lang="ts">
  import { goto } from "$app/navigation";
  import { KosoError } from "$lib/api";
  import { Button } from "$lib/components/ui/button";
  import Navbar from "$lib/navbar.svelte";
  import * as rest from "$lib/projects";
  import { type Project, type ProjectExport } from "$lib/projects";
  import { HardDriveUpload, Layers, PackagePlus, Trash2 } from "lucide-svelte";
  import { toast } from "svelte-sonner";

  let deflicker: Promise<Project[]> = new Promise((r) => setTimeout(r, 50));
  let projects: Promise<Project[]> = rest.fetchProjects();

  async function createProject(projectExport: ProjectExport | null = null) {
    const toastId = toast.loading(
      projectExport
        ? `Importing project ${projectExport.projectId}...`
        : `Creating project...`,
    );
    try {
      let project = await rest.createProject(projectExport);
      await goto(`/projects/${project.projectId}`);
      toast.success(projectExport ? "Project imported!" : "Project created!", {
        id: toastId,
      });
    } catch (err) {
      if (err instanceof KosoError && err.hasReason("TOO_MANY_PROJECTS")) {
        toast.error(
          "Cannot create new project, you already have too many. Contact us for more!",
          { id: toastId, duration: 10000 },
        );
      } else if (err instanceof KosoError && err.status === 422) {
        toast.error(
          "The Koso export file is malformed. Verify the correct file was selected and try again.",
          { id: toastId, duration: 10000 },
        );
      } else {
        console.warn(err);
        toast.error("Something went wrong. Please try again.", {
          id: toastId,
          duration: 10000,
        });
      }
      return;
    }
  }

  function triggerFileSelect() {
    document.getElementById("projectImportFileInput")?.click();
  }

  async function deleteProject(project: Project) {
    const toastId = toast.loading(`Moving ${project.name} to the trash...`);
    try {
      await rest.deleteProject(project);
      projects = rest.fetchProjects();
      toast.success(
        `${project.name} has been placed in the trash and will be permanently deleted in 30 days.`,
        { id: toastId },
      );
    } catch (err) {
      if (err instanceof KosoError) {
        toast.error(
          `Could not move ${project.name} to the trash: ${err.message}`,
          { id: toastId, duration: 10000 },
        );
      }
      console.warn(err);
      toast.error("Something went wrong. Please try again.", {
        id: toastId,
        duration: 10000,
      });
    }
  }

  function parseProjectExport(data: string) {
    try {
      return JSON.parse(data);
    } catch (e) {
      toast.error(
        "The Koso export file is malformed. Verify the correct file was selected and try again.",
        { duration: 10000 },
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

    event.currentTarget.value = "";

    if (files.length > 1) {
      toast.error("Select a single file.", { duration: 10000 });
      return;
    }

    let projectExport = parseProjectExport(await file.text());
    await createProject(projectExport);
  }
</script>

<Navbar />

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
          <PackagePlus />
          New
        </Button>
        <Button title="Import Project" onclick={triggerFileSelect}>
          <HardDriveUpload />
          Import
        </Button>
      </div>
    </div>
  {:else}
    <div class="m-2 flex flex-col rounded border">
      <div class="flex flex-col items-end p-2">
        <div>
          <Button title="New Project" onclick={() => createProject()}>
            <PackagePlus />
            New
          </Button>
          <Button title="Import Project" onclick={triggerFileSelect}>
            <HardDriveUpload />
            Import
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
              <Trash2 />
              Trash
            </Button>
          </div>
        {/each}
      </div>
    </div>
  {/if}
{/await}
