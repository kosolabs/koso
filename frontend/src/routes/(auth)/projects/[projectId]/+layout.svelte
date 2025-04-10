<script lang="ts">
  import { nav } from "$lib/nav.svelte";
  import { fetchProject, fetchProjectUsers } from "$lib/projects";
  import { type Snippet } from "svelte";
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
