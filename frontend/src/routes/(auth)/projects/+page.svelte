<script lang="ts">
  import { goto } from "$app/navigation";
  import { token, user } from "$lib/auth";
  import { Alert } from "$lib/components/ui/alert";
  import { Button } from "$lib/components/ui/button";
  import Navbar from "$lib/navbar.svelte";
  import {
    fetchProjects,
    createProject as projectsCreateProject,
    type Project,
  } from "$lib/projects";
  import { Layers } from "lucide-svelte";
  import { onMount } from "svelte";

  let projects: Promise<Project[]> = new Promise(() => {});
  let errorMessage: string | null = null;

  async function createProject() {
    errorMessage = null;
    let project;
    try {
      project = await projectsCreateProject($token);
    } catch (err) {
      errorMessage = `${err}`;
      return;
    }
    await goto(`/projects/${project.project_id}`);
  }

  onMount(() => {
    if (!$user) {
      return;
    }

    projects = fetchProjects($token);
  });
</script>

<Navbar />

{#if errorMessage}
  <div class="my-2 flex-grow-0">
    <Alert variant="destructive">{errorMessage}</Alert>
  </div>
{/if}

{#await projects}
  <div class="flex flex-col items-center justify-center rounded border p-4">
    <div class="text-xl">Loading...</div>
  </div>
{:then projects}
  {#if projects.length === 0}
    <div class="flex flex-col items-center justify-center rounded border p-4">
      <div class="mb-2"><Layers /></div>
      <div class="mb-4 text-xl">Create your first Koso project!</div>
      <div>
        <Button on:click={() => createProject()}>New project</Button>
      </div>
    </div>
  {:else}
    <div class="flex flex-col rounded border">
      <div class="flex flex-col items-end bg-slate-100 p-2 dark:bg-slate-900">
        <div>
          <Button on:click={() => createProject()}>New project</Button>
        </div>
      </div>
      <div
        class="flex flex-col items-stretch [&>*:nth-child(even)]:bg-slate-50 [&>*:nth-child(even)]:dark:bg-slate-950"
      >
        {#each projects as project}
          <div class="border-t p-2">
            <Button
              class="text-lg"
              variant="link"
              href="projects/{project.project_id}"
            >
              {project.name}
            </Button>
          </div>
        {/each}
      </div>
    </div>
  {/if}
{/await}
