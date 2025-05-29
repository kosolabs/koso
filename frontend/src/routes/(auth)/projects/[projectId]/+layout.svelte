<script lang="ts">
  import { page } from "$app/state";
  import { getAuthContext, showUnauthorizedDialog } from "$lib/auth.svelte";
  import {
    getRegistryContext,
    type ActionID,
  } from "$lib/components/ui/command-palette";
  import { getPrefsContext } from "$lib/components/ui/prefs";
  import { ProjectShareModal } from "$lib/components/ui/project-share-modal";
  import { redirectToGithubInstallFlow } from "$lib/github";
  import { Action } from "$lib/kosui/command";
  import { nav } from "$lib/nav.svelte";
  import { NavigationAction } from "$lib/navigation-action";
  import { fetchProject, fetchProjectUsers } from "$lib/projects";
  import {
    Eye,
    Mail,
    Notebook,
    PanelTopClose,
    PanelTopOpen,
    Pencil,
    UserPlus,
  } from "lucide-svelte";
  import { onMount, type Snippet } from "svelte";
  import { newProjectContext } from "../../../../lib/dag-table/project-context.svelte";

  type Props = {
    children: Snippet;
  };
  let { children }: Props = $props();

  let openShareModal: boolean = $state(false);

  const authCtx = getAuthContext();
  const ctx = newProjectContext(authCtx);
  const command = getRegistryContext();
  const prefs = getPrefsContext();
  nav.lastVisitedProjectId = ctx.id;
  const deflicker: Promise<void> = new Promise((r) => window.setTimeout(r, 50));
  const loading = load();

  async function load() {
    const [project, users] = await Promise.all([
      fetchProject(authCtx, ctx.id),
      fetchProjectUsers(authCtx, ctx.id),
    ]);
    ctx.name = project.name;
    ctx.users = users;
  }

  $effect(() => {
    if (ctx.socket.unauthorized) {
      showUnauthorizedDialog();
    }
  });

  const actions: Action<ActionID>[] = [
    new NavigationAction({
      id: "InboxView",
      href: `/projects/${ctx.id}/inbox`,
      title: "Zero inbox",
      description: "Navigate to Zero Inbox view",
      icon: Mail,
    }),
    new NavigationAction({
      id: "PlanView",
      href: `/projects/${ctx.id}`,
      title: "Project planning",
      description: "Navigate to Project Planning view",
      icon: Notebook,
    }),
    new Action({
      id: "DetailPanelClose",
      callback: () => (prefs.detailPanel = "none"),
      title: "Close task description",
      description: "Close / hide the task description markdown panel",
      icon: PanelTopClose,
      enabled: () => prefs.detailPanel !== "none",
    }),
    new Action({
      id: "DetailPanelOpen",
      callback: () => (prefs.detailPanel = "view"),
      title: "Open task description",
      description: "Open / show the task description markdown panel",
      icon: PanelTopOpen,
      enabled: () => prefs.detailPanel === "none",
    }),
    new Action({
      id: "DetailPanelViewer",
      callback: () => (prefs.detailPanel = "view"),
      title: "View task description",
      description: "Open / show the task description markdown viewer",
      icon: Eye,
      enabled: () => prefs.detailPanel !== "view",
    }),
    new Action({
      id: "DetailPanelEditor",
      callback: () => (prefs.detailPanel = "edit"),
      title: "Edit task description",
      description: "Open / show the task description markdown editor",
      icon: Pencil,
      enabled: () => prefs.detailPanel !== "edit",
    }),
    new Action({
      id: "ConnectToGitHub",
      callback: async () =>
        await redirectToGithubInstallFlow(authCtx, ctx.id, page.url.pathname),
      title: "Connect to GitHub",
      description: "Connect the project to GitHub",
      icon: UserPlus,
    }),
    new Action({
      id: "ShareProject",
      callback: () => (openShareModal = true),
      title: "Share project",
      description: "Open / show the project share dialog",
      icon: UserPlus,
      enabled: () => !!authCtx.fullUser?.premium,
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
  {#if !!authCtx.fullUser?.premium}
    <ProjectShareModal bind:open={openShareModal} />
  {/if}

  {@render children()}
{/await}
