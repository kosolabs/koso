<script lang="ts">
  import { page } from "$app/stores";
  import { logout as auth_logout, token, user, type User } from "$lib/auth";
  import { DagTable } from "$lib/DagTable";
  import { Koso } from "$lib/koso";
  import { disableRedirectOnLogOut, lastVisitedProjectId } from "$lib/nav";
  import Navbar from "$lib/navbar.svelte";
  import { fetchProjects, type Project, updateProject } from "$lib/projects";
  import { A, Button, Input, Label, Modal } from "flowbite-svelte";
  import { UserPlus } from "lucide-svelte";
  import { onDestroy, onMount } from "svelte";
  import * as Y from "yjs";

  const projectId = $page.params.slug;
  const koso = new Koso(projectId, new Y.Doc());

  let users: User[] = [];
  let project: Project | null = null;

  let shareModal = false;

  async function logout() {
    disableRedirectOnLogOut();
    auth_logout();
  }

  async function loadUsers() {
    if (!$user || !$token) throw new Error("User is unauthorized");

    let resp = await fetch(`/api/projects/${projectId}/users`, {
      headers: { Authorization: "Bearer " + $token },
    });
    if (!resp.ok) {
      throw new Error(
        `Failed to fetch project users: ${resp.statusText} (${resp.status})`,
      );
    }

    return await resp.json();
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

  let socket: WebSocket | null = null;

  onMount(async () => {
    if (!$user || !$token) {
      return;
    }

    [users, project] = await Promise.all([loadUsers(), loadProject()]);

    // const response = await fetch(`/api/projects/${projectId}/permissions`, {
    //   method: "PATCH",
    //   headers: {
    //     Authorization: `Bearer ${$token}`,
    //     "Content-Type": "application/json",
    //   },
    //   body: JSON.stringify({
    //     project_id: projectId,
    //     add_emails: [
    //       "examplee@gmail.com",
    //       "example1@gmail.com",
    //       "example2@gmail.com",
    //     ],
    //     remove_emails: ["shadanan@gmail.com", "leonhard.kyle@gmail.com"],
    //   }),
    // });
    // if (!response.ok) {
    //   throw new Error(
    //     `Failed to update project name: ${response.statusText} (${response.status})`,
    //   );
    // }

    const host = location.origin.replace(/^http/, "ws");
    const wsUrl = `${host}/api/ws/projects/${projectId}`;
    socket = new WebSocket(wsUrl, ["bearer", $token]);
    socket.binaryType = "arraybuffer";

    socket.onopen = () => {
      koso.handleClientMessage((update) => {
        if (socket) {
          socket.send(update);
        }
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
      console.log("WebSocket errored", event);
      // Error type is not available, so assume unauthorized and logout
      $lastVisitedProjectId = null;
      logout();
    };
    socket.onclose = (event) => {
      console.log(
        `WebSocket closed. Code: ${event.code}, Reason: '${event.reason}''`,
        event,
      );
      socket = null;
    };

    while (socket && socket.readyState == WebSocket.CONNECTING) {
      await new Promise((r) => setTimeout(r, 100));
    }
  });

  onDestroy(() => {
    if (socket) {
      socket.close(1000, "Closed in onDestroy.");
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
    <Button
      size="xs"
      title="Share Project"
      on:click={() => (shareModal = true)}
    >
      <UserPlus />
    </Button>
  </svelte:fragment>
</Navbar>

<DagTable {koso} {users} />

<Modal
  title="Share your project"
  bind:open={shareModal}
  autoclose
  outsideclose
>
  <form class="flex flex-col space-y-6" action="#">
    <Label class="space-y-2">
      <span>Collaborators</span>
      <Input
        type="email"
        name="email"
        placeholder="name@company.com"
        required
      />
    </Label>

    {#each users as user}
      {user.name} -- {user.email}
    {/each}

    <Button type="submit" class="w-full1">Share</Button>
  </form>
</Modal>
