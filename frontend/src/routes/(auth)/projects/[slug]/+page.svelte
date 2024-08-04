<script lang="ts">
  import { page } from "$app/stores";
  import { logout as auth_logout, token, user } from "$lib/auth";
  import { DagTable } from "$lib/DagTable";
  import { Koso } from "$lib/koso";
  import { disableRedirectOnLogOut, lastVisitedProjectId } from "$lib/nav";
  import Navbar from "$lib/navbar.svelte";
  import {
    fetchProjects,
    type Project,
    type ProjectUsers,
    updateProject,
  } from "$lib/projects";
  import { A, Button, Input } from "flowbite-svelte";
  import { UserPlus } from "lucide-svelte";
  import { onMount } from "svelte";
  import * as Y from "yjs";

  const projectId = $page.params.slug;
  const koso = new Koso(projectId, new Y.Doc());

  let projectUsers: ProjectUsers = {};
  let project: Project | null = null;

  async function logout() {
    disableRedirectOnLogOut();
    auth_logout();
  }

  async function updateProjectUsers() {
    if (!$user || !$token) throw new Error("User is unauthorized");

    let resp = await fetch(`/api/projects/${projectId}/users`, {
      headers: { Authorization: "Bearer " + $token },
    });
    if (!resp.ok) {
      throw new Error(
        `Failed to fetch project users: ${resp.statusText} (${resp.status})`,
      );
    }

    const projectUsers: ProjectUsers = {};
    for (const user of await resp.json()) {
      projectUsers[user.email] = user;
    }
    return projectUsers;
  }

  async function loadProject() {
    if (!$user || !$token) throw new Error("User is unauthorized");

    const projects = await fetchProjects($token);
    for (const project of projects) {
      if (project.project_id == projectId) {
        return project;
      }
    }
    throw new Error(
      `Project ${projectId} does not exist or user no longer has access: ${projects})`,
    );
  }

  let editedProjectName: string | null = null;

  function handleStartEditingProjectName(event: MouseEvent | CustomEvent) {
    event.stopPropagation();
    editedProjectName = project?.name || "";
  }

  async function saveEditedProjectName() {
    if (!editedProjectName) {
      editedProjectName = null;
      return;
    }
    if (!$user || !$token) throw new Error("User is unauthorized");

    const updatedProject = await updateProject($token, {
      project_id: projectId,
      name: editedProjectName,
    });

    if (project) {
      project.name = updatedProject.name;
    }
    editedProjectName = null;
  }

  function revertEditedProjectName() {
    if (editedProjectName === null) {
      return;
    }
    editedProjectName = null;
  }

  async function handleEditedProjectNameBlur() {
    await saveEditedProjectName();
  }

  async function handleEditedProjectNameKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      revertEditedProjectName();
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    if (event.key === "Enter") {
      await saveEditedProjectName();
      event.preventDefault();
      event.stopPropagation();
      return;
    }
  }

  onMount(async () => {
    if (!$user || !$token) {
      return;
    }

    [projectUsers, project] = await Promise.all([
      updateProjectUsers(),
      loadProject(),
    ]);

    const host = location.origin.replace(/^http/, "ws");
    const wsUrl = `${host}/ws/projects/${projectId}`;
    const socket = new WebSocket(wsUrl, ["bearer", $token]);
    socket.binaryType = "arraybuffer";

    socket.onopen = () => {
      koso.handleClientMessage((update) => {
        socket.send(update);
      });
      $lastVisitedProjectId = $page.params.slug;
    };

    socket.onmessage = (event) => {
      if (event.data instanceof ArrayBuffer) {
        koso.handleServerMessage(new Uint8Array(event.data));
      } else {
        console.log("Received text frame from server:", event.data);
      }
    };
    socket.onerror = (event) => {
      console.log(event);
      // Error type is not available, so assume unauthorized and logout
      $lastVisitedProjectId = null;
      logout();
    };

    while (socket.readyState !== WebSocket.OPEN) {
      await new Promise((r) => setTimeout(r, 100));
    }
  });
</script>

<Navbar>
  <svelte:fragment slot="left-items">
    <div>
      {#if editedProjectName !== null}
        <Input
          size="lg"
          class="ml-2 p-1"
          on:click={(event) => event.stopPropagation()}
          on:blur={handleEditedProjectNameBlur}
          on:keydown={handleEditedProjectNameKeydown}
          bind:value={editedProjectName}
          autofocus
        />
      {:else if project}
        <A
          class="ml-2 hover:no-underline"
          on:click={handleStartEditingProjectName}
          on:keydown={handleStartEditingProjectName}
        >
          {project.name}
        </A>
      {/if}
    </div>
  </svelte:fragment>
  <svelte:fragment slot="right-items">
    <Button size="xs" title="Share Project"><UserPlus /></Button>
  </svelte:fragment>
</Navbar>

<DagTable {koso} {projectUsers} />
