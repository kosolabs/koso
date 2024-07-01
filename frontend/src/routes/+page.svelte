<script lang="ts">
  import { DagTable, Path, type Graph } from "$lib/DagTable";

  let tasks: Graph = {
    "000": { id: "000", title: "Root", children: ["001", "004", "005"] },
    "001": { id: "001", title: "Task 1", children: ["002"] },
    "002": { id: "002", title: "Task 1.1", children: ["003"] },
    "003": { id: "003", title: "Task 1.1.1", children: [] },
    "004": { id: "004", title: "Task 2", children: [] },
    "005": { id: "005", title: "Task 3", children: [] },
  };
</script>

<DagTable bind:graph={tasks} root={new Path(["000"])} />
