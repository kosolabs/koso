<script lang="ts">
  import { KosoError } from "$lib/api";
  import { getAuthContext } from "$lib/auth.svelte";
  import { DetailPanel } from "$lib/components/ui/detail-panel";
  import { Editable } from "$lib/components/ui/editable";
  import { Navbar } from "$lib/components/ui/navbar";
  import { getPrefsContext } from "$lib/components/ui/prefs";
  import { toast } from "$lib/components/ui/sonner";
  import { Toolbar } from "$lib/components/ui/toolbar";
  import {
    DagTable,
    getProjectContext,
    newPlanningContext,
    OfflineAlert,
  } from "$lib/dag-table";
  import { updateProject } from "$lib/projects";

  const prefs = getPrefsContext();
  const auth = getAuthContext();
  const project = getProjectContext();
  const { koso } = project;
  const planningCtx = newPlanningContext(koso);

  let detailPanel: DetailPanel | undefined = $state();
  let offline: boolean = $derived(project.socket.offline);

  async function saveEditedProjectName(name: string) {
    let updatedProject;
    try {
      updatedProject = await updateProject(auth, {
        projectId: project.id,
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
    project.name = updatedProject.name;
  }
</script>

<div class="flex h-dvh flex-col">
  <div class="grow-0">
    <Navbar breadcrumbs={["Projects", project.name]}>
      {#snippet left()}
        <div>
          <Editable
            class="ml-2 text-lg"
            value={project.name}
            aria-label="Set project name"
            onsave={async (name) => {
              await saveEditedProjectName(name);
            }}
            onkeydown={(e) => e.stopPropagation()}
          />
        </div>
      {/snippet}
    </Navbar>

    <OfflineAlert {offline} />
  </div>

  <div class="grow overflow-hidden p-1">
    <div class="flex h-full flex-row-reverse max-2xl:flex-col">
      {#if prefs.detailPanel !== "none"}
        <div class="flex-1 overflow-y-scroll p-1">
          <DetailPanel
            bind:this={detailPanel}
            taskId={planningCtx.selected?.name}
          />
        </div>
      {/if}
      <div class="flex-2 overflow-y-scroll p-1">
        <DagTable users={project.users} />
      </div>
    </div>
  </div>

  <div class="sm:hidden">
    <Toolbar selected={!!planningCtx.selected} />
  </div>
</div>
