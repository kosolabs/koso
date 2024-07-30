<script lang="ts">
  import { page } from "$app/stores";
  import kosoLogo from "$lib/assets/koso.svg";
  import { logout as auth_logout, token, user } from "$lib/auth";
  import { DagTable } from "$lib/DagTable";
  import { Koso } from "$lib/koso";
  import { disableRedirectOnLogOut, lastVisitedProjectId } from "$lib/nav";
  import {
    Button,
    Navbar,
    NavBrand,
    NavHamburger,
    NavLi,
    NavUl,
  } from "flowbite-svelte";
  import NavContainer from "flowbite-svelte/NavContainer.svelte";
  import * as decoding from "lib0/decoding";
  import * as encoding from "lib0/encoding";
  import { UserPlus } from "lucide-svelte";
  import { onMount } from "svelte";
  import * as syncProtocol from "y-protocols/sync";
  import * as Y from "yjs";

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

    socket.onopen = (event) => {
      const encoder = encoding.createEncoder();
      encoding.writeVarInt(encoder, 0);
      syncProtocol.writeSyncStep1(encoder, koso.yDoc);
      socket.send(encoding.toUint8Array(encoder));

      koso.onLocalUpdate((update) => {
        const encoder = encoding.createEncoder();
        encoding.writeVarInt(encoder, 0);
        syncProtocol.writeUpdate(encoder, update);
        socket.send(encoding.toUint8Array(encoder));
      });
      $lastVisitedProjectId = $page.params.slug;
    };

    socket.onmessage = (event) => {
      if (event.data instanceof ArrayBuffer) {
        console.log("Received: ", new Uint8Array(event.data));
        const decoder = decoding.createDecoder(new Uint8Array(event.data));
        const messageProtocolType = decoding.readVarUint(decoder);
        if (messageProtocolType !== 0) {
          throw new Error(
            `Expected protocol message type to be Sync (0) but was: ${messageProtocolType}`,
          );
        }

        const encoder = encoding.createEncoder();
        encoding.writeVarUint(encoder, 0);
        const emptyLength = encoding.length(encoder);
        syncProtocol.readSyncMessage(decoder, encoder, koso.yDoc, "");

        if (encoding.length(encoder) > emptyLength) {
          const reply = encoding.toUint8Array(encoder);
          console.log("Sending: ", reply);
          socket.send(reply);
        }
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

<Navbar color="primary" class="mb-4" fluid={true}>
  <NavContainer fluid={true}>
    <NavBrand>
      <img class="w-14" alt="Koso Logo" src={kosoLogo} />
    </NavBrand>
    <div class="flex md:order-2">
      <Button size="xs"><UserPlus /></Button>
      <NavHamburger />
    </div>
    <NavUl slideParams={{ delay: 0, duration: 250 }}>
      <NavLi href="/projects">Projects</NavLi>
      <NavLi on:click={() => logout()}>Logout</NavLi>
    </NavUl>
  </NavContainer>
</Navbar>
<DagTable {koso} />
