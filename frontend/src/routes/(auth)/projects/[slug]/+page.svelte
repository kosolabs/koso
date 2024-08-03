<script lang="ts">
  import { page } from "$app/stores";
  import { logout as auth_logout, token, user } from "$lib/auth";
  import { DagTable } from "$lib/DagTable";
  import { NavBar } from "$lib/NavBar";
  import { Koso } from "$lib/koso";
  import { disableRedirectOnLogOut, lastVisitedProjectId } from "$lib/nav";
  import type { ProjectUsers } from "$lib/projects";
  import { Button } from "flowbite-svelte";
  import { UserPlus } from "lucide-svelte";
  import { onMount } from "svelte";
  import * as Y from "yjs";

  const projectId = $page.params.slug;
  const koso = new Koso(projectId, new Y.Doc());

  let projectUsers: ProjectUsers = {};
  $: console.log(projectUsers);

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

  onMount(async () => {
    if (!$user || !$token) {
      return;
    }

    projectUsers = await updateProjectUsers();

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

<NavBar>
  <Button slot="nav-items" size="xs" title="Share Project"><UserPlus /></Button>
</NavBar>

<DagTable {koso} {projectUsers} />
