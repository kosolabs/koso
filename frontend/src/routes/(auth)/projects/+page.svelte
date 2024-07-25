<script lang="ts">
  import { token, user } from "$lib/auth";
  import { onMount } from "svelte";
  import { Alert, Button, A } from "flowbite-svelte";
  import { goto } from "$app/navigation";
  import { createProject, fetchProjects, type Project } from "$lib/projects";

  let projects: Promise<Project[]> = new Promise(() => {});
  let errorMessage: string | null = null;

  async function createProjectt() {
    errorMessage = null;
    let project;
    try {
      project = await createProject($token);
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

<div class="flex flex-col rounded border p-4">
  {#await projects}
    <div>Loading...</div>
  {:then projects}
    {#if projects.length === 0}
      <div class="m-auto mb-8 text-xl">Create your first Koso project!</div>
      <div class="m-auto flex items-end gap-2">
        <div>
          <Button on:click={() => createProjectt()}>New project</Button>
        </div>
        {#if errorMessage}
          <Alert class="mt-8" border>{errorMessage}</Alert>
        {/if}
      </div>
    {:else}
      <div>
        <div>
          <Button on:click={() => createProjectt()}>New project</Button>
        </div>
        {#if errorMessage}
          <Alert class="mt-8" border>{errorMessage}</Alert>
        {/if}
      </div>
      <div>
        {#each projects as project}
          <div><A href="projects/{project.project_id}">{project.name}</A></div>
        {/each}
      </div>
    {/if}
  {/await}
</div>
