<script lang="ts">
  import { KosoError } from "$lib/api";
  import {
    getRegistryContext,
    type ActionID,
  } from "$lib/components/ui/command-palette";
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
  import { Action } from "$lib/kosui/command";
  import { exportProject, updateProject } from "$lib/projects";
  import { FileDown } from "lucide-svelte";
  import { onMount } from "svelte";

  const command = getRegistryContext();
  const project = getProjectContext();
  const { koso } = project;
  const planningCtx = newPlanningContext(koso);
  const prefs = getPrefsContext();

  async function saveEditedProjectName(name: string) {
    let updatedProject;
    try {
      updatedProject = await updateProject({ projectId: project.id, name });
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

  async function exportProjectToFile() {
    toast.info("Exporting project...");
    const projectExport = await exportProject(project.id);

    let projectName = (project.name || "project")
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

  const actions: Action<ActionID>[] = [
    new Action({
      id: "ExportProject",
      callback: exportProjectToFile,
      title: "Export project",
      description: "Export project to JSON",
      icon: FileDown,
    }),
  ];

  onMount(() => {
    return command.register(...actions);
  });
</script>

<div class="flex h-dvh flex-col">
  <div class="grow-0">
    <Navbar>
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

    <OfflineAlert offline={project.socket.offline} />
  </div>

  <div class="grow overflow-hidden p-1">
    <div class="flex h-full flex-row-reverse max-2xl:flex-col">
      {#if prefs.detailPanel !== "none"}
        <div class="flex-1 overflow-y-scroll p-1">
          <DetailPanel taskId={planningCtx.selected?.name} />
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
