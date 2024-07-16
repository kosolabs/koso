<script lang="ts">
  import { DagTable, type Graph } from "$lib/DagTable";

  import { onMount } from "svelte";
  import * as Y from "yjs";

  let graph: Graph = {};

  const yDoc = new Y.Doc();
  const yGraph = yDoc.getMap<Y.Map<string | Y.Array<string>>>("graph");

  yGraph.observeDeep(() => {
    graph = yGraph.toJSON();
  });

  onMount(async () => {
    const host = location.origin.replace(/^http/, "ws");
    // TODO: Get project id from the path.
    const socket = new WebSocket(`${host}/ws/projects/koso-staging`);
    socket.binaryType = "arraybuffer";
    socket.addEventListener("message", function (event) {
      if (event.data instanceof ArrayBuffer) {
        Y.applyUpdate(yDoc, new Uint8Array(event.data));
      } else {
        console.log("Received text frame from server:", event.data);
      }
    });

    while (socket.readyState !== WebSocket.OPEN) {
      await new Promise((r) => setTimeout(r, 100));
    }

    yDoc.on("update", (update) => {
      socket.send(update);
    });
  });

  function addNode(nodeId: string, parentId: string, offset: number) {
    yDoc.transact(() => {
      const yParent = yGraph.get(parentId)!;
      const yChildren = yParent.get("children") as Y.Array<string>;
      yChildren.insert(offset, [nodeId]);
    });
  }

  function removeNode(nodeId: string, parentId: string) {
    yDoc.transact(() => {
      const yParent = yGraph.get(parentId)!;
      const yChildren = yParent.get("children") as Y.Array<string>;
      yChildren.delete(yChildren.toArray().indexOf(nodeId));
    });
  }

  function moveNode(
    nodeId: string,
    srcParentId: string,
    srcOffset: number,
    destParentId: string,
    destOffset: number,
  ) {
    yDoc.transact(() => {
      const ySrcParent = yGraph.get(srcParentId)!;
      const ySrcChildren = ySrcParent.get("children") as Y.Array<string>;
      ySrcChildren.delete(srcOffset);

      const yDestParent = yGraph.get(destParentId)!;
      const yDestChildren = yDestParent.get("children") as Y.Array<string>;
      if (srcParentId === destParentId && srcOffset < destOffset) {
        destOffset -= 1;
      }
      yDestChildren.insert(destOffset, [nodeId]);
    });
  }

  function editTaskName(taskId: string, newName: string) {
    yDoc.transact(() => {
      const yNode = yGraph.get(taskId)!;
      if (yNode.get("name") !== newName) {
        yNode.set("name", newName);
      }
    });
  }
</script>

<DagTable {graph} {addNode} {removeNode} {moveNode} {editTaskName} />
