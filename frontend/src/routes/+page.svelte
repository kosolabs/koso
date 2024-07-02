<script lang="ts">
  import { DagTable, type Graph } from "$lib/DagTable";
  import { onMount } from "svelte";

  let graph: Graph = {};

  async function update() {
    const resp = await fetch("/api/tasks");
    const tasks = await resp.json();
    for (const task of tasks) {
      graph[task["id"]] = { ...task };
    }
  }

  onMount(async () => {
    await update();

    const host = location.origin.replace(/^http/, "ws");
    const socket = new WebSocket(`${host}/ws`);
    socket.addEventListener("message", function (event) {
      console.log("Message from server ", event.data);
    });
  });
</script>

<DagTable bind:graph />
