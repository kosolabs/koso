<script lang="ts" context="module">
  export type SplicedNode = {
    node: Node;
    offset: number;
  };

  export type Interactions = {
    dragged: Node | null;
    ghost: SplicedNode | null;
    highlighted: Node | null;
  };

  export type TableContext = {
    addNode: (nodeId: string, parentId: string, offset: number) => void;
    removeNode: (nodeId: string, parentId: string) => void;
    moveNode: (
      nodeId: string,
      srcParentId: string,
      destParentId: string,
      offset: number,
    ) => void;
    editTaskName: (taskId: string, newName: string) => void;
    setDragged: (node: Node) => void;
    clearDragged: () => void;
    setGhost: (node: Node, offset: number) => void;
    clearGhost: () => void;
    setHighlighted: (node: Node) => void;
    clearHighlighted: () => void;
  };
</script>

<script lang="ts">
  import { setContext } from "svelte";
  import { Node, type Graph } from ".";
  import Row from "./row.svelte";

  export let graph: Graph;
  export let addNode: (
    nodeId: string,
    parentId: string,
    offset: number,
  ) => void;
  export let removeNode: (nodeId: string, parentId: string) => void;
  export let moveNode: (
    nodeId: string,
    srcParentId: string,
    destParentId: string,
    offset: number,
  ) => void;
  export let editTaskName: (taskId: string, newName: string) => void;

  function findRoots(graph: Graph): Node[] {
    const allChildren = new Set<string>();
    for (const node of Object.values(graph)) {
      for (const child of node.children) {
        allChildren.add(child);
      }
    }
    const allNodeIds = new Set<string>(Object.keys(graph));
    const rootIds = allNodeIds.difference(allChildren);
    return Array.from(rootIds).map((rootId) => new Node([rootId]));
  }

  $: roots = findRoots(graph);

  setContext<TableContext>("graph", {
    addNode,
    removeNode,
    moveNode,
    editTaskName,
    setDragged: (node: Node) => {
      interactions.dragged = node;
      interactions = interactions;
    },
    clearDragged: () => {
      interactions.dragged = null;
      interactions = interactions;
    },
    setGhost: (node: Node, offset: number) => {
      interactions.ghost = { node, offset };
      interactions = interactions;
    },
    clearGhost: () => {
      interactions.ghost = null;
      interactions = interactions;
    },
    setHighlighted: (node: Node) => {
      interactions.highlighted = node;
      interactions = interactions;
    },
    clearHighlighted: () => {
      interactions.highlighted = null;
      interactions = interactions;
    },
  });

  let interactions: Interactions = {
    dragged: null,
    ghost: null,
    highlighted: null,
  };
</script>

<h1 class="my-8 text-4xl">Yotei Hierarchical Table</h1>

<div>
  <div id="header" class="rounded border text-xs font-bold uppercase">
    <div class="my-1 flex items-center rounded p-2">
      <div class="border-r" style="width: 12rem">
        <div class="flex items-center">
          <div class="w-5"></div>
          <div class="w-5"></div>
          <div>ID</div>
        </div>
      </div>
      <div class="w-96 px-2">Description</div>
    </div>
  </div>

  <div
    id="body"
    class="[&>*:nth-child(even)]:bg-slate-50 [&>*:nth-child(odd)]:bg-slate-100"
  >
    {#each roots as root}
      <Row {graph} {interactions} node={root} />
    {/each}
  </div>
</div>
