<script lang="ts">
  import { page } from "$app/stores";
  import kosoLogo from "$lib/assets/koso.svg";
  import { logout, token } from "$lib/auth";
  import { DagTable } from "$lib/DagTable";
  import { Koso } from "$lib/koso";
  import { A } from "flowbite-svelte";
  import { onMount } from "svelte";
  import * as Y from "yjs";

  const koso = new Koso(new Y.Doc());

  onMount(async () => {
    if (!$token) {
      return;
    }
    const host = location.origin.replace(/^http/, "ws");
    const wsUrl = `${host}/ws/projects/${$page.params.slug}`;
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
      logout();
    };

    while (socket.readyState !== WebSocket.OPEN) {
      await new Promise((r) => setTimeout(r, 100));
    }

    koso.onLocalUpdate((update) => {
      socket.send(update);
    });
  });
</script>

<A class="m-auto mb-4" href="/">
  <img class="w-10" alt="Koso Logo" src={kosoLogo} />
  <div class="ml-2 text-4xl">Koso</div>
</A>
<DagTable {koso} />