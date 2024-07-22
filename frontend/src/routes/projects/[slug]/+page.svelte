<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { token } from "$lib/auth";
  import { DagTable } from "$lib/DagTable";
  import { Koso } from "$lib/DagTable/koso";
  import { A } from "flowbite-svelte";
  import { onMount } from "svelte";
  import * as Y from "yjs";

  const kosoGraph = new Koso(new Y.Doc());

  onMount(async () => {
    const projectId = $page.params.slug;
    if ($token === null) {
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

    kosoGraph.onLocalUpdate((update) => {
      socket.send(update);
    });
  });
</script>

<A href="/"><h1 class="mb-4 text-2xl">Koso Home</h1></A>
<DagTable koso={kosoGraph} />
