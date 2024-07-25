<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import kosoLogo from "$lib/assets/koso.svg";
  import { logout as auth_logout, token, user } from "$lib/auth";
  import { DagTable } from "$lib/DagTable";
  import { Koso } from "$lib/koso";
  import {
    Button,
    Navbar,
    NavBrand,
    NavUl,
    NavLi,
    NavHamburger,
  } from "flowbite-svelte";
  import NavContainer from "flowbite-svelte/NavContainer.svelte";
  import { UserPlus } from "lucide-svelte";
  import { onMount } from "svelte";
  import * as Y from "yjs";
  import { disableRedirectOnLogOut, lastVisitedProjectId } from "$lib/nav";

  const projectId = $page.params.slug;
  const koso = new Koso(projectId, new Y.Doc());

  async function logout() {
    disableRedirectOnLogOut();
    auth_logout();
  }

  onMount(async () => {
    if (!$user || !$token) {
      return;
    }

    const host = location.origin.replace(/^http/, "ws");
    const wsUrl = `${host}/ws/projects/${projectId}`;
    const socket = new WebSocket(wsUrl, ["bearer", $token]);
    socket.binaryType = "arraybuffer";
    socket.onmessage = (event) => {
      if (event.data instanceof ArrayBuffer) {
        koso.update(new Uint8Array(event.data));
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

    $lastVisitedProjectId = $page.params.slug;

    koso.onLocalUpdate((update) => {
      socket.send(update);
    });
  });
</script>

<Navbar color="primary" class="mb-4">
  <NavContainer>
    <NavBrand>
      <img class="w-14" alt="Koso Logo" src={kosoLogo} />
    </NavBrand>
    <div class="flex md:order-2">
      <Button size="xs"><UserPlus /></Button>
      <NavHamburger />
      <NavUl>
        <NavLi on:click={() => goto("/projects")}>Projects</NavLi>
        <NavLi on:click={() => logout()}>Logout</NavLi>
      </NavUl>
    </div>
  </NavContainer>
</Navbar>
<DagTable {koso} />
