<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { showUnauthorizedDialog } from "$lib/auth.svelte";
  import { command, type ActionID } from "$lib/components/ui/command-palette";
  import { getPrefsContext } from "$lib/components/ui/prefs";
  import { ProjectShareModal } from "$lib/components/ui/project-share-modal";
  import { githubInstallUrl } from "$lib/github";
  import { Action } from "$lib/kosui/command";
  import { nav } from "$lib/nav.svelte";
  import { fetchProject, fetchProjectUsers } from "$lib/projects";
  import {
    Mail,
    Notebook,
    PanelTopClose,
    PanelTopOpen,
    SquarePen,
    UserPlus,
  } from "lucide-svelte";
  import { onMount, type Snippet } from "svelte";
  import { newProjectContext } from "../../../../lib/dag-table/project-context.svelte";

  type Props = {
    children: Snippet;
  };
  let { children }: Props = $props();

  let openShareModal: boolean = $state(false);

  const ctx = newProjectContext();
  const prefs = getPrefsContext();
  nav.lastVisitedProjectId = ctx.id;
  const deflicker: Promise<void> = new Promise((r) => window.setTimeout(r, 50));
  const loading = load();

  async function load() {
    const [project, users] = await Promise.all([
      fetchProject(ctx.id),
      fetchProjectUsers(ctx.id),
    ]);
    ctx.name = project.name;
    ctx.users = users;
    ctx.premium = users.some((u) => u.premium);
  }

  $effect(() => {
    if (ctx.socket.unauthorized) {
      showUnauthorizedDialog();
    }
  });

  const actions: Action<ActionID>[] = [
    new Action({
      id: "InboxView",
      callback: () => goto(`/projects/${ctx.id}/inbox`),
      title: "Zero inbox",
      description: "Navigate to Zero Inbox view",
      icon: Mail,
      enabled: () => page.url.pathname !== `/projects/${ctx.id}/inbox`,
    }),
    new Action({
      id: "PlanView",
      callback: () => goto(`/projects/${ctx.id}`),
      title: "Project planning",
      description: "Navigate to Project Planning view",
      icon: Notebook,
      enabled: () => page.url.pathname !== `/projects/${ctx.id}`,
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
      title: "View task description",
      description: "Open / show the task description markdown viewer",
      icon: PanelTopOpen,
      enabled: () => prefs.detailPanel !== "view",
    }),
    new Action({
      id: "DetailPanelEditor",
      callback: () => (prefs.detailPanel = "edit"),
      title: "Edit task description",
      description: "Open / show the task description markdown editor",
      icon: SquarePen,
      enabled: () => prefs.detailPanel !== "edit",
    }),
    new Action({
      id: "ConnectToGitHub",
      callback: async () =>
        window.location.assign(await githubInstallUrl(ctx.id)),
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
      enabled: () => ctx.premium,
    }),
  ];

  onMount(() => {
    return command.register(...actions);
  });
</script>

{#if ctx.premium}
  <ProjectShareModal bind:open={openShareModal} />
{/if}

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
  {@render children()}
{/await}
