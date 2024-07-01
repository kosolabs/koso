<script lang="ts">
  import { DagTable, Path, type Graph } from "$lib/DagTable";
  import { onMount } from "svelte";

  let graph: Graph = {
    "000": { id: "000", title: "Root", children: ["001", "004", "005"] },
    "001": { id: "001", title: "Task 1", children: ["002"] },
    "002": { id: "002", title: "Task 1.1", children: ["003"] },
    "003": { id: "003", title: "Task 1.1.1", children: [] },
    "004": { id: "004", title: "Task 2", children: [] },
    "005": { id: "005", title: "Task 3", children: [] },
  };
  let root = new Path(["000"]);

  async function update() {
    const resp = await fetch("/task/stream");
    const tasks = await resp.json();
    for (const task of tasks) {
      graph[task["id"]] = {
        id: task["id"],
        title: task["name"],
        children: task["children"],
      };
    }
    console.log(graph);
    root = new Path(["6f98a57c-0ccc-4551-a1c9-e528eefd7252"]);
  }

  onMount(() => {
    update();
  });
</script>

<DagTable bind:graph {root} />
