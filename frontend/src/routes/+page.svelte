<script lang="ts">
  import { goto } from "$app/navigation";
  import { token } from "$lib/auth";
  import { DagTable } from "$lib/DagTable";
  import { KosoGraph } from "$lib/DagTable/kosograph";

  import { onMount } from "svelte";
  import * as Y from "yjs";

  const kosoGraph = new KosoGraph(new Y.Doc());

  onMount(async () => {
    if ($token === null) {
      return await goto("/login");
    }
    const host = location.origin.replace(/^http/, "ws");
    // TODO: Get project id from the path.
    const socket = new WebSocket(`${host}/ws/projects/koso-staging`, [
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

<DagTable {kosoGraph} />
