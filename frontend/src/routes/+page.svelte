<script lang="ts">
  import { DagTable, type Graph } from "$lib/DagTable";
  import { Button } from "flowbite-svelte";

  import { onMount } from "svelte";
  import * as Y from "yjs";

  let graph: Graph = {};

  const yDoc = new Y.Doc();
  const yGraph = yDoc.getMap<Y.Map<string | Y.Array<string>>>("graph");

  yGraph.observeDeep(() => {
    graph = yGraph.toJSON();
    console.log(JSON.stringify(yGraph.toJSON()));
  });

  onMount(async () => {
    const host = location.origin.replace(/^http/, "ws");
    const socket = new WebSocket(`${host}/ws`);
    socket.binaryType = "arraybuffer";
    socket.addEventListener("message", function (event) {
      console.log("Message from server", new Uint8Array(event.data));
      Y.applyUpdate(yDoc, new Uint8Array(event.data));
    });

    while (socket.readyState !== WebSocket.OPEN) {
      await new Promise((r) => setTimeout(r, 100));
    }

    yDoc.on("update", (update) => {
      console.log("Sending update", update);
      socket.send(update);
    });
  });

  function toY(
    id: string,
    name: string,
    children: string[],
  ): Y.Map<string | Y.Array<string>> {
    const yChildren = new Y.Array<string>();
    yChildren.insert(0, children);
    const yNode = new Y.Map<string | Y.Array<string>>();
    yNode.set("id", id);
    yNode.set("name", name);
    yNode.set("children", yChildren);
    return yNode;
  }

  function load() {
    yDoc.transact(() => {
      yGraph.set("0", toY("0", "Root", ["1", "4", "5"]));
      yGraph.set("1", toY("1", "Task 1", ["2"]));
      yGraph.set("2", toY("2", "Task 1.1", ["3"]));
      yGraph.set("3", toY("3", "Task 1.1.1", []));
      yGraph.set("4", toY("4", "Task 2", []));
      yGraph.set("5", toY("5", "Task 3", ["6"]));
      yGraph.set("6", toY("6", "Task 6", []));
    });
  }

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
    destParentId: string,
    offset: number,
  ) {
    yDoc.transact(() => {
      const ySrcParent = yGraph.get(srcParentId)!;
      const ySrcChildren = ySrcParent.get("children") as Y.Array<string>;
      ySrcChildren.delete(ySrcChildren.toArray().indexOf(nodeId));

      const yDestParent = yGraph.get(destParentId)!;
      const yDestChildren = yDestParent.get("children") as Y.Array<string>;
      yDestChildren.insert(offset, [nodeId]);
    });
  }
</script>

<Button on:click={() => load()}>Load</Button>

<DagTable {graph} {addNode} {removeNode} {moveNode} />
