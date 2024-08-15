<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { writable } from "svelte/store";
  import { token, user, type User } from "$lib/auth";
  import { DagTable } from "$lib/DagTable";
  import { Koso } from "$lib/koso";
  import { lastVisitedProjectId } from "$lib/nav";
  import Navbar from "$lib/navbar.svelte";
  import ProjectShare from "$lib/project-share.svelte";
  import { type ShareState } from "$lib/project-share.svelte";
  import { get } from "svelte/store";
  import {
    fetchProjects,
    type Project,
    updateProject,
    fetchProjectUsers,
  } from "$lib/projects";

  import { A, Alert, Button, Input, Label, Modal } from "flowbite-svelte";
  import { UserPlus, CircleMinus, TriangleAlert } from "lucide-svelte";
  import { createEventDispatcher, onDestroy, onMount } from "svelte";
  import * as Y from "yjs";

  const projectId = $page.params.slug;
  const koso = new Koso(projectId, new Y.Doc());
  window.koso = koso;

  let project: Project | null = null;

  let shareModalState: ShareState = {
    open: false,
    projectId: projectId,
    projectUsers: writable<User[]>([]),
  };
  $: projectUsers = shareModalState.projectUsers;

  async function loadProjectUsers() {
    if (!$user || !$token) throw new Error("User is unauthorized");
    return await fetchProjectUsers($token, projectId);
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
  let showSocketOfflineAlert: boolean = false;
  let showUnauthorizedModal: boolean = false;

  async function openWebSocket(backoffMs = 1000) {
    if (!$user || !$token) throw new Error("User is unauthorized");

    const host = location.origin.replace(/^http/, "ws");
    const wsUrl = `${host}/api/ws/projects/${projectId}`;
    socket = new WebSocket(wsUrl, ["bearer", $token]);
    socket.binaryType = "arraybuffer";

    socket.onopen = (event) => {
      console.log("WebSocket opened", event);
      showSocketOfflineAlert = false;
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
      // Errors also trigger onclose events so handle everything there.
    };
    socket.onclose = (event) => {
      if (socket === null && event.code === 1000) {
        console.log(
          `WebSocket closed in onDestroy. Code: ${event.code}, Reason: '${event.reason}' Will not try to reconnect`,
          event,
        );
        return;
      }

      const UNAUTHORIZED = 3000;
      if (event.code === UNAUTHORIZED) {
        console.log(
          `Unauthorized, WebSocket closed. Code: ${event.code}, Reason: '${event.reason}'. `,
          event,
        );
        // Don't redirect the user back to a project they don't have access too.
        $lastVisitedProjectId = null;
        socket = null;
        showUnauthorizedModal = true;
        return;
      }

      const OVERLOADED = 1013;
      if (event.code === OVERLOADED) {
        // In case of overload, don't retry aggressively.
        if (backoffMs < 30000) {
          backoffMs = 30000;
        }
        console.log(
          `Overloaded WebSocket closed. Code: ${event.code}, Reason: '${event.reason}'. Will try to reconnect in ${backoffMs} ms.`,
          event,
        );
      } else {
        console.log(
          `WebSocket closed. Code: ${event.code}, Reason: '${event.reason}'. Will try to reconnect in ${backoffMs} ms.`,
          event,
        );
      }

      showSocketOfflineAlert = true;
      setTimeout(async () => {
        if (socket !== null) {
          await openWebSocket(Math.min(backoffMs * 2, 60000));
        }
      }, backoffMs);
    };

    while (socket && socket.readyState == WebSocket.CONNECTING) {
      await new Promise((r) => setTimeout(r, 100));
    }
  }

  onMount(async () => {
    if (!$user || !$token) {
      return;
    }

    let [projectUsersT, projectT] = await Promise.all([
      loadProjectUsers(),
      loadProject(),
      openWebSocket(),
    ]);
    shareModalState.projectUsers.update((pu) => {
      pu.push(...projectUsersT);
      return pu;
    });
    shareModalState.projectUsers = shareModalState.projectUsers;
    project = projectT;
  });

  onDestroy(() => {
    if (socket) {
      if (socket) {
        socket.close(1000, "Closed in onDestroy.");
      }
      socket = null;
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
      on:click={() => {
        shareModalState.open = true;
      }}
    >
      <UserPlus />
    </Button>
  </svelte:fragment>
</Navbar>

{#if showSocketOfflineAlert}
  <div class="mt-4">
    <Alert class="border">Connection to server lost. Working offline.</Alert>
  </div>
{/if}

<Modal title="Unauthorized" bind:open={showUnauthorizedModal}>
  <p class="text-base leading-relaxed text-gray-500 dark:text-gray-400">
    You do not have access to the project or the project does not exist.
  </p>
  <svelte:fragment slot="footer">
    <Button on:click={() => goto("/projects")}>Take me home</Button>
  </svelte:fragment>
</Modal>

<DagTable {koso} users={$projectUsers} />

<!-- TODO: Figure out how to modify projectUsers passed to dag table on add/remove. -->
<ProjectShare state={shareModalState} />
