<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { getAuthContext, showUnauthorizedDialog } from "$lib/auth.svelte";
  import { getRegistryContext } from "$lib/components/ui/command-palette";
  import {
    ActionIds,
    Categories,
  } from "$lib/components/ui/command-palette/command-palette.svelte";
  import { getPrefsContext } from "$lib/components/ui/prefs";
  import { ProjectShareModal } from "$lib/components/ui/project-share-modal";
  import { toast } from "$lib/components/ui/sonner";
  import { redirectToGithubInstallFlow } from "$lib/github";
  import { nav } from "$lib/nav.svelte";
  import { NavigationAction } from "$lib/navigation-action";
  import {
    exportProject,
    fetchProject,
    fetchProjectUsers,
  } from "$lib/projects";
  import {
    CircleGauge,
    Eye,
    FileDown,
    Github,
    Mail,
    Notebook,
    PanelTopClose,
    PanelTopOpen,
    Pencil,
    UserPlus,
  } from "@lucide/svelte";
  import { Action } from "kosui";
  import { onMount, type Snippet } from "svelte";
  import { newProjectContext } from "../../../../lib/dag-table/project-context.svelte";

  type Props = {
    children: Snippet;
  };
  let { children }: Props = $props();

  let openShareModal: boolean = $state(false);

  const auth = getAuthContext();
  const ctx = newProjectContext(auth);
  const command = getRegistryContext();
  const prefs = getPrefsContext();
  nav.lastVisitedProjectId = ctx.id;
  const deflicker: Promise<void> = new Promise((r) => window.setTimeout(r, 50));
  const loading = load();

  async function exportProjectToFile() {
    toast.info("Exporting project...");
    const projectExport = await exportProject(auth, ctx.id);

    let projectName = (ctx.name || "project")
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

  async function load() {
    const [project, users] = await Promise.all([
      fetchProject(auth, ctx.id),
      fetchProjectUsers(auth, ctx.id),
    ]);
    ctx.name = project.name;
    ctx.users = users;
  }

  function getNextIterationDashboard() {
    const plannedTasks = ctx.koso.getCurrentIterations();
    if (plannedTasks.length > 0) {
      return `/projects/${ctx.id}/dash/${plannedTasks[0].id}`;
    }
  }

  $effect(() => {
    if (ctx.socket.unauthorized) {
      showUnauthorizedDialog();
    }
  });

  const actions: Action[] = [
    new NavigationAction({
      id: ActionIds.InboxView,
      href: `/projects/${ctx.id}/inbox`,
      category: Categories.Navigation,
      name: "Zero Inbox",
      description: "Navigate to Zero Inbox view",
      icon: Mail,
    }),
    new NavigationAction({
      id: ActionIds.PlanView,
      href: `/projects/${ctx.id}`,
      category: Categories.Navigation,
      name: "Project Planning",
      description: "Navigate to Project Planning view",
      icon: Notebook,
    }),
    new Action({
      id: ActionIds.NextDashView,
      callback: () => goto(getNextIterationDashboard()!),
      category: Categories.Navigation,
      name: "Current Iteration",
      description: "Navigate to current iteration's dashboard view",
      icon: CircleGauge,
      enabled: () => !!getNextIterationDashboard(),
    }),
    new Action({
      id: ActionIds.DetailPanelClose,
      callback: () => (prefs.detailPanel = "none"),
      category: Categories.MarkdownPanel,
      name: "Close Markdown Panel",
      description: "Close / hide the task description markdown panel",
      icon: PanelTopClose,
      enabled: () => prefs.detailPanel !== "none",
    }),
    new Action({
      id: ActionIds.DetailPanelOpen,
      callback: () => (prefs.detailPanel = "view"),
      category: Categories.MarkdownPanel,
      name: "Open Markdown Panel",
      description: "Open / show the task description markdown panel",
      icon: PanelTopOpen,
      enabled: () => prefs.detailPanel === "none",
    }),
    new Action({
      id: ActionIds.DetailPanelViewer,
      callback: () => (prefs.detailPanel = "view"),
      category: Categories.MarkdownPanel,
      name: "Show Markdown Viewer",
      description: "Open / show the task description markdown viewer",
      icon: Eye,
      enabled: () => prefs.detailPanel !== "view",
    }),
    new Action({
      id: ActionIds.DetailPanelEditor,
      callback: () => (prefs.detailPanel = "edit"),
      category: Categories.MarkdownPanel,
      name: "Show Markdown Editor",
      description: "Open / show the task description markdown editor",
      icon: Pencil,
      enabled: () => prefs.detailPanel !== "edit",
    }),
    new Action({
      id: ActionIds.ConnectToGitHub,
      callback: async () =>
        await redirectToGithubInstallFlow(auth, ctx.id, page.url.toString()),
      category: Categories.Project,
      name: "Connect to GitHub",
      description: "Connect the project to GitHub",
      icon: Github,
    }),
    new Action({
      id: ActionIds.ShareProject,
      callback: () => (openShareModal = true),
      category: Categories.Project,
      name: "Share Project...",
      description: "Open / show the project share dialog",
      icon: UserPlus,
      enabled: () => !!auth.fullUser?.premium,
    }),
    new Action({
      id: ActionIds.ExportProject,
      callback: exportProjectToFile,
      category: Categories.Project,
      name: "Export Project to JSON",
      description: "Export Project to JSON",
      icon: FileDown,
    }),
  ];

  onMount(() => {
    return command.register(...actions);
  });
</script>

{#await loading}
  {#await deflicker}
    <!-- Deflicker load. -->
  {:then}
    <!-- TODO: Make this a Skeleton -->
    <div class="flex flex-col items-center justify-center rounded border p-4">
      <div class="text-l">Loading...</div>
    </div>
  {/await}
{:then}
  {#if !!auth.fullUser?.premium}
    <ProjectShareModal bind:open={openShareModal} />
  {/if}

  {@render children()}
{/await}
