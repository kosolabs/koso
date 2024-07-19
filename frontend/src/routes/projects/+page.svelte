<script lang="ts">
  import { goto } from "$app/navigation";
  import { token } from "$lib/auth";
  import { DagTable } from "$lib/DagTable";
  import { KosoGraph } from "$lib/DagTable/kosograph";
  import { A } from "flowbite-svelte";
  import { onMount } from "svelte";
  import * as Y from "yjs";

  const kosoGraph = new KosoGraph(new Y.Doc());

  onMount(async () => {
    const urlParams = new URLSearchParams(window.location.search);
    const projectId = urlParams.get("project");
    if ($token === null || projectId === null) {
      return await goto("/");
    }
    const host = location.origin.replace(/^http/, "ws");
    // TODO: Get project id from the path.
    const socket = new WebSocket(`${host}/ws/projects/${projectId}`, [
      "bearer",
      $token,
    ]);
    socket.binaryType = "arraybuffer";
    socket.onmessage = (event) => {
      if (event.data instanceof ArrayBuffer) {
        kosoGraph.update(new Uint8Array(event.data));
      } else {
        console.log("Received text frame from server:", event.data);
      }
    };

    while (socket.readyState !== WebSocket.OPEN) {
      await new Promise((r) => setTimeout(r, 100));
    }

    kosoGraph.onupdate((update) => {
      socket.send(update);
    });
  });
</script>

<A href="/"><h1 class="mb-4 text-2xl">Koso Home</h1></A>
<DagTable {kosoGraph} />
