<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { token, user, type User } from "$lib/auth";
  import { Alert } from "$lib/components/ui/alert";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { DagTable } from "$lib/dag-table";
  import { Koso } from "$lib/koso";
  import { lastVisitedProjectId } from "$lib/nav";
  import Navbar from "$lib/navbar.svelte";
  import ProjectShareModal from "$lib/project-share-modal.svelte";
  import {
    fetchProjects,
    fetchProjectUsers,
    type Project,
    updateProject,
  } from "$lib/projects";
  import { UserPlus } from "lucide-svelte";
  import { onDestroy, onMount } from "svelte";
  import * as Y from "yjs";

  const projectId = $page.params.projectId;
  const koso = new Koso(projectId, new Y.Doc());
  window.koso = koso;

  let project: Project | null = null;
  let projectUsers: User[] = [];
  let openShareModal = false;

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

  function handleStartEditingProjectName(event: MouseEvent | KeyboardEvent) {
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

  let showSocketOfflineAlert: boolean = false;
  let showUnauthorizedModal: boolean = false;

  class KosoSocket {
    socket: WebSocket | null = null;
    shutdown: boolean = false;
    socketPingInterval: ReturnType<typeof setTimeout> | null = null;
    reconnectBackoffMs: number | null = null;
    offlineTimeout: ReturnType<typeof setTimeout> | null = null;
    offlineHandler: (event: Event) => void;
    onlineHandler: (event: Event) => Promise<void>;

    constructor() {
      this.setOffline();

      this.onlineHandler = this.handleOnline.bind(this);
      window.addEventListener("online", this.onlineHandler);
      this.offlineHandler = this.handleOffline.bind(this);
      window.addEventListener("offline", this.offlineHandler);
      this.socketPingInterval = setInterval(
        () => {
          if (this.socket && this.socket.readyState == WebSocket.OPEN) {
            this.socket.send("");
          }
        },
        (45 + 20 * Math.random()) * 1000,
      );
    }

    async openWebSocket() {
      if (!$user || !$token) throw new Error("User is unauthorized");
      if (
        this.socket &&
        (this.socket.readyState == WebSocket.OPEN ||
          this.socket.readyState == WebSocket.CONNECTING)
      ) {
        console.log("Socket already connected");
        return;
      }
      if (this.shutdown) {
        return;
      }

      const host = location.origin.replace(/^http/, "ws");
      const wsUrl = `${host}/api/ws/projects/${projectId}`;
      const socket = new WebSocket(wsUrl, ["bearer", $token]);
      this.socket = socket;
      socket.binaryType = "arraybuffer";

      socket.onopen = (event) => {
        console.log("WebSocket opened", event);
        this.setOnline();
        koso.handleClientMessage((update) => {
          if (socket.readyState == WebSocket.OPEN) {
            socket.send(update);
          } else {
            console.warn(
              "Tried to send to non-open socket, discarded message",
              socket,
            );
          }
        });
        $lastVisitedProjectId = $page.params.projectId;
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
        // Sometimes onclose events are delayed while, in the meantime,
        // a new socket was opened.
        if (this.socket && this.socket != socket) {
          console.log("Socket already reopened");
          return;
        }

        this.setOffline();
        if (this.shutdown) {
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
          this.setShutdown();
          showUnauthorizedModal = true;
          return;
        }

        const OVERLOADED = 1013;
        let backoffMs;
        if (event.code === OVERLOADED) {
          // In case of overload, don't retry aggressively.
          backoffMs = this.backoffOnReconnect(30000);
          console.log(
            `Overloaded WebSocket closed. Code: ${event.code}, Reason: '${event.reason}'. Will try to reconnect in ${backoffMs} ms.`,
            event,
          );
        } else {
          backoffMs = this.backoffOnReconnect();
          console.log(
            `WebSocket closed. Code: ${event.code}, Reason: '${event.reason}'. Will try to reconnect in ${backoffMs} ms.`,
            event,
          );
        }

        // Try to reconnect.
        setTimeout(async () => {
          await this.openWebSocket();
        }, backoffMs);
      };

      while (socket.readyState == WebSocket.CONNECTING) {
        await new Promise((r) => setTimeout(r, 100));
      }
    }

    async handleOnline(event: Event) {
      console.log("Online.", event);
      await this.openWebSocket();
    }

    handleOffline(event: Event) {
      console.log("Offline.", event);
      this.setOffline(1);
      if (this.socket) {
        const socket = this.socket;
        this.socket = null;
        socket.close(1000, "Went offline");
      }
    }

    closeAndShutdown(code: number, reason: string) {
      const socket = this.socket;
      this.setShutdown();
      if (socket) {
        socket.close(code, reason);
      }
    }

    setShutdown() {
      this.shutdown = true;
      if (this.socketPingInterval) {
        clearInterval(this.socketPingInterval);
      }
      if (this.offlineTimeout) {
        clearTimeout(this.offlineTimeout);
      }
      window.removeEventListener("online", this.onlineHandler);
      window.removeEventListener("offline", this.offlineHandler);
      this.socket = null;
    }

    setOffline(alertDelayMs = 14000) {
      if (!this.offlineTimeout && !this.shutdown) {
        // Delay showing the offline alert for a little bit
        // to avoid flashing an alert due to transient events.
        // e.g. server restarts.
        this.offlineTimeout = setTimeout(() => {
          if (this.offlineTimeout && !this.shutdown) {
            showSocketOfflineAlert = true;
          }
        }, alertDelayMs);
      }
    }

    setOnline() {
      this.reconnectBackoffMs = null;
      if (this.offlineTimeout) {
        clearTimeout(this.offlineTimeout);
        this.offlineTimeout = null;
      }
      showSocketOfflineAlert = false;
    }

    backoffOnReconnect(min: number = 0): number {
      let base = this.reconnectBackoffMs ? this.reconnectBackoffMs * 1.5 : 400;
      // Don't let backoff get too big (or too small).
      base = Math.max(Math.min(60000, base), min);
      // Add some jitter
      this.reconnectBackoffMs = base + base * 0.3 * Math.random();
      return this.reconnectBackoffMs;
    }
  }

  const kosoSocket = new KosoSocket();

  onMount(async () => {
    if (!$user || !$token) {
      return;
    }

    [projectUsers, project] = await Promise.all([
      loadProjectUsers(),
      loadProject(),
      kosoSocket.openWebSocket(),
    ]);
  });

  onDestroy(() => {
    kosoSocket.closeAndShutdown(1000, "Closed in onDestroy.");
  });
</script>

<Navbar>
  <svelte:fragment slot="left-items">
    <div>
      {#if editedProjectName !== null}
        <Input
          class="ml-2 p-2"
          on:click={(event) => event.stopPropagation()}
          on:blur={handleEditedProjectNameBlur}
          on:keydown={handleEditedProjectNameKeydown}
          bind:value={editedProjectName}
          autofocus
        />
      {:else if project}
        <Button
          data-testid="set-project-name-button"
          variant="link"
          class="text-lg"
          on:click={handleStartEditingProjectName}
          on:keydown={handleStartEditingProjectName}
        >
          {project.name}
        </Button>
      {/if}
    </div>
  </svelte:fragment>
  <svelte:fragment slot="right-items">
    <Button
      title="Share Project"
      on:click={() => {
        openShareModal = true;
      }}
    >
      <UserPlus />
    </Button>
  </svelte:fragment>
</Navbar>

{#if showSocketOfflineAlert}
  <div class="m-4">
    <Alert>Connection to server lost. Working offline.</Alert>
  </div>
{/if}

<Dialog.Root bind:open={showUnauthorizedModal}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Unauthorized</Dialog.Title>
      <Dialog.Description>
        You do not have access to the project or the project does not exist.
      </Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer>
      <Button on:click={() => goto("/projects")}>Take me home</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<ProjectShareModal bind:open={openShareModal} bind:projectUsers {project} />

<DagTable {koso} users={projectUsers} />
