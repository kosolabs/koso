<script lang="ts">
  import { goto } from "$app/navigation";
  import { token, user } from "$lib/auth";
  import Navbar from "$lib/navbar.svelte";
  import {
    fetchProjects,
    createProject as projectsCreateProject,
    type Project,
  } from "$lib/projects";
  import { A, Alert, Button } from "flowbite-svelte";
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
      {#if errorMessage}
        <div class="mt-4">
          <Alert class="border">{errorMessage}</Alert>
        </div>
      {/if}
    </div>
  {:else}
    <div class="flex flex-col rounded border">
      <div class="roundedborder flex flex-col items-end bg-slate-100 p-2">
        <div>
          <Button on:click={() => createProject()}>New project</Button>
        </div>
        {#if errorMessage}
          <div class="mt-4 flex-grow-0">
            <Alert class="border">{errorMessage}</Alert>
          </div>
        {/if}
      </div>
      <div
        class="flex flex-col items-stretch [&>*:nth-child(even)]:bg-slate-50"
      >
        {#each projects as project}
          <div class="rounded border p-2">
            <A href="projects/{project.project_id}">{project.name}</A>
          </div>
        {/each}
      </div>
    </div>
  {/if}
{/await}
