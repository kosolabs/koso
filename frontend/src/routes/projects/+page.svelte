<script lang="ts">
  import { token } from "$lib/auth";
  import { onMount } from "svelte";
  import { Alert, Avatar, Button, Input, Label } from "flowbite-svelte";
  import { goto } from "$app/navigation";

  type Project = {
    project_id: string;
    name: string;
  };

  let projects: Promise<Project[]> = new Promise(() => {});
  let projectName: String;

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
    const projectId = projectName.toLowerCase().replaceAll(/ |_/g, "-");
    // TODO: validate stuff
    const response = await fetch("/api/projects", {
      method: "POST",
      headers: {
        Authorization: `Bearer ${$token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ project_id: projectId, name: projectName }),
    });
    if (!response.ok) {
      console.log("failed create", response);
      // TODO: Handle errors
    } else {
      goto(`projects/${projectId}`);
    }
  }

  onMount(() => {
    projects = fetchProjects();
  });
</script>

<div class="flex flex-col rounded border p-4">
  {#await projects}
    loading
  {:then projects}
    {#if projects.length === 0}
      <div class="m-auto mb-8 text-xl">Create your first Koso project!</div>
    {:else}
      <div>
        {#each projects as project}
          <div>{project.project_id}: {project.name}</div>
        {/each}
      </div>
    {/if}
    <div class="m-auto flex items-end gap-2">
      <div>
        <Input
          type="text"
          id="name"
          bind:value={projectName}
          placeholder="My Amazing Project"
          aria-label="Project Name"
          required
        />
      </div>
      <div>
        <Button on:click={() => createProject()}>New project</Button>
      </div>
    </div>
  {/await}
</div>
