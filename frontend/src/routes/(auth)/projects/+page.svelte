<script lang="ts">
  import { goto } from "$app/navigation";
  import { token, user } from "$lib/auth";
  import { A, Button } from "$lib/button";
  import { Alert } from "$lib/components/ui/alert";
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
  <!-- TODO: Make this a Skeleton -->
  <div class="flex flex-col items-center justify-center rounded border p-4">
    <div class="text-xl">Loading...</div>
  </div>
{:then projects}
  {#if projects.length === 0}
    <div
      class="m-4 flex flex-col items-center gap-6 rounded border bg-card p-8"
    >
      <div><Layers /></div>
      <div class="text-xl">Create your first Koso project!</div>
      <div>
        <Button on:click={() => createProject()}>New project</Button>
      </div>
    </div>
  {:else}
    <div class="m-4 flex flex-col rounded border">
      <div class="flex flex-col items-end p-2">
        <div>
          <Button on:click={() => createProject()}>New project</Button>
        </div>
      </div>
      <div
        class="flex flex-col items-stretch [&>*:nth-child(even)]:bg-row-even [&>*:nth-child(odd)]:bg-row-odd"
      >
        {#each projects as project}
          <div class="border-t p-2">
            <A class="text-lg" href="projects/{project.project_id}">
              {project.name}
            </A>
          </div>
        {/each}
      </div>
    </div>
  {/if}
{/await}
