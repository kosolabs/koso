<script lang="ts">
  import { logout, token, user } from "$lib/auth";
  import { onMount } from "svelte";
  import {
    Avatar,
    Alert,
    Button,
    A,
    Navbar,
    NavBrand,
    NavHamburger,
    Dropdown,
    DropdownItem,
    DropdownHeader,
  } from "flowbite-svelte";
  import NavContainer from "flowbite-svelte/NavContainer.svelte";
  import { goto } from "$app/navigation";
  import {
    createProject as projectsCreateProject,
    fetchProjects,
    type Project,
  } from "$lib/projects";
  import kosoLogo from "$lib/assets/koso.svg";
  import { Layers, UserPlus } from "lucide-svelte";

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

<Navbar color="primary" class="mb-4" fluid={true}>
  <NavContainer fluid={true}>
    <NavBrand href="/projects">
      <img class="w-14" alt="Koso Logo" src={kosoLogo} />
    </NavBrand>
    <div class="flex md:order-2">
      <Button size="xs" title="Share Project"><UserPlus /></Button>
      <Button
        id="profile-menu"
        class="ms-3 rounded-full border bg-slate-200 p-2"
        title="Profile"
      >
        <div><Avatar src={$user?.picture} size="xs" /></div>
      </Button>
      <Dropdown triggeredBy="#profile-menu">
        <DropdownHeader>
          <span class="block text-sm">{$user?.name}</span>
          <span class="block truncate text-sm font-medium">{$user?.email}</span>
        </DropdownHeader>
        <DropdownItem href="/projects">Projects</DropdownItem>
        <DropdownItem on:click={() => logout()}>Logout</DropdownItem>
      </Dropdown>
      <NavHamburger />
    </div>
  </NavContainer>
</Navbar>

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
