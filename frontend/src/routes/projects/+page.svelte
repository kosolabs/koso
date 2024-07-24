<script lang="ts">
  import { token } from "$lib/auth";
  import { onMount } from "svelte";
  import { Alert, Avatar, Button, Input, Label, A } from "flowbite-svelte";
  import { goto } from "$app/navigation";

  type Project = {
    project_id: string;
    name: string;
  };

  let projects: Promise<Project[]> = new Promise(() => {});
  let errorMessage: string | null = null;

  async function fetchProjects() {
    const response = await fetch("/api/projects", {
      method: "GET",
      headers: {
        Authorization: `Bearer ${$token}`,
      },
    });
    return await response.json();
  }

  async function createProject() {
    errorMessage = null;

    const response = await fetch("/api/projects", {
      method: "POST",
      headers: {
        Authorization: `Bearer ${$token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ name: "My Project!" }),
    });
    if (!response.ok) {
      console.log("failed create", response);
      errorMessage = `Failed to create project: ${response.statusText} (${response.status})`;
    } else {
      const project: Project = await response.json();
      goto(`projects/${project.project_id}`);
    }
  }

  onMount(() => {
    projects = fetchProjects();
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
          <Button on:click={() => createProject()}>New project</Button>
        </div>
        {#if errorMessage}
          <Alert class="mt-8" border>{errorMessage}</Alert>
        {/if}
      </div>
    {:else}
      <div>
        <div>
          <Button on:click={() => createProject()}>New project</Button>
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
