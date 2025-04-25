<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { showUnauthorizedDialog } from "$lib/auth.svelte";
  import { command, type ActionID } from "$lib/components/ui/command-palette";
  import { Action } from "$lib/kosui/command";
  import { nav } from "$lib/nav.svelte";
  import { fetchProject, fetchProjectUsers } from "$lib/projects";
  import { Mail, Notebook } from "lucide-svelte";
  import { onMount, type Snippet } from "svelte";
  import { newProjectContext } from "../../../../lib/dag-table/project-context.svelte";

  type Props = {
    children: Snippet;
  };
  let { children }: Props = $props();

  const ctx = newProjectContext();
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
      title: "Zero Inbox",
      description: "Navigate to Zero Inbox view",
      icon: Mail,
      enabled: () => page.url.pathname !== `/projects/${ctx.id}/inbox`,
    }),
    new Action({
      id: "PlanView",
      callback: () => goto(`/projects/${ctx.id}`),
      title: "Project Planning",
      description: "Navigate to Project Planning view",
      icon: Notebook,
      enabled: () => page.url.pathname !== `/projects/${ctx.id}`,
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
  {@render children()}
{/await}
