<script lang="ts">
  import { goto } from "$app/navigation";
  import { KosoError } from "$lib/api";
  import { command, type ActionID } from "$lib/components/ui/command-palette";
  import { Editable } from "$lib/components/ui/editable";
  import { Navbar } from "$lib/components/ui/navbar";
  import { toast } from "$lib/components/ui/sonner";
  import {
    DagTable,
    getProjectContext,
    newPlanningContext,
    OfflineAlert,
  } from "$lib/dag-table";
  import { githubInstallUrl } from "$lib/github";
  import { baseClasses } from "$lib/kosui/base";
  import { Action } from "$lib/kosui/command";
  import { Menu, MenuContent, MenuItem, MenuTrigger } from "$lib/kosui/menu";
  import { exportProject, updateProject } from "$lib/projects";
  import { cn } from "$lib/utils";
  import { FileDown, Mail, MenuIcon, PlugZap, UserPlus } from "lucide-svelte";
  import { onMount } from "svelte";
  import ProjectShareModal from "../project-share-modal.svelte";

  const project = getProjectContext();
  const { koso } = project;
  newPlanningContext(koso);
  let openShareModal: boolean = $state(false);

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
      title: "Export Project",
      description: "Export project to JSON",
      icon: FileDown,
    }),
    new Action({
      id: "ShareProject",
      callback: () => (openShareModal = true),
      title: "Share project",
      description: "Open / show the project share dialog",
      icon: UserPlus,
    }),
  ];

  onMount(() => {
    return command.register(...actions);
  });
</script>

<ProjectShareModal bind:open={openShareModal} />

<div class="flex h-dvh flex-col">
  <div class="grow-0">
    <Navbar>
      {#snippet context()}
        <Menu>
          <MenuTrigger
            title="Project menu"
            class={cn(
              baseClasses({
                variant: "plain",
                color: "primary",
                shape: "circle",
                focus: true,
                hover: true,
              }),
              "mr-1 p-2 transition-all active:scale-95",
            )}
          >
            <MenuIcon size={20} />
          </MenuTrigger>
          <MenuContent>
            <MenuItem
              class="gap-2"
              onSelect={async () =>
                window.location.assign(await githubInstallUrl(project.id))}
            >
              <PlugZap size={16} />
              Connect to GitHub
            </MenuItem>
            <MenuItem class="gap-2" onSelect={exportProjectToFile}>
              <FileDown size={16} />
              Export project
            </MenuItem>
            <MenuItem
              class="gap-2"
              onSelect={() => goto(`/projects/${project.id}/inbox`)}
            >
              <Mail size={16} />
              Navigate to Zero Inbox
            </MenuItem>
            <MenuItem
              class="gap-2"
              onSelect={() => {
                openShareModal = true;
              }}
            >
              <UserPlus size={16} />
              Share project
            </MenuItem>
          </MenuContent>
        </Menu>
      {/snippet}
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

  <div class="p-2">
    <DagTable users={project.users} />
  </div>
</div>
