<script lang="ts">
  import { DagTable, Path, type Graph } from "$lib/DagTable";
  import { onMount } from "svelte";

  let graph: Graph = {
    "000": { id: "000", name: "Root", children: ["001", "004", "005"] },
    "001": { id: "001", name: "Task 1", children: ["002"] },
    "002": { id: "002", name: "Task 1.1", children: ["003"] },
    "003": { id: "003", name: "Task 1.1.1", children: [] },
    "004": { id: "004", name: "Task 2", children: [] },
    "005": { id: "005", name: "Task 3", children: [] },
  };
  let root = new Path(["000"]);

  async function update() {
    const resp = await fetch("/api/tasks");
    const tasks = await resp.json();
    for (const task of tasks) {
      graph[task["id"]] = { ...task };
    }
    root = new Path(["6f98a57c-0ccc-4551-a1c9-e528eefd7252"]);
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

<DagTable bind:graph {root} />
