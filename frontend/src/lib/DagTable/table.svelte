<script lang="ts" context="module">
  export type Interactions = {
    dragged: Path | null;
    ghost: { parentId: string; childId: string; offset: number } | null;
    maybePeer: Path | null;
    maybeChild: Path | null;
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
    setDragged: (node: Path) => void;
    clearDragged: () => void;
    setMaybePeer: (node: Path) => void;
    setMaybeChild: (node: Path) => void;
    clearMaybePeerAndChild: () => void;
  };
</script>

<script lang="ts">
  import { setContext } from "svelte";
  import { Path, type Graph } from ".";
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

  function findRoots(graph: Graph): Path[] {
    const allChildren = new Set<string>();
    for (const node of Object.values(graph)) {
      for (const child of node.children) {
        allChildren.add(child);
      }
    }
    const allNodeIds = new Set<string>(Object.keys(graph));
    const rootIds = allNodeIds.difference(allChildren);
    return Array.from(rootIds).map((rootId) => new Path([rootId]));
  }

  $: roots = findRoots(graph);

  setContext<TableContext>("graph", {
    addNode,
    removeNode,
    moveNode,
    editTaskName,
    setDragged: (node: Path) => {
      interactions.dragged = node;
      interactions = interactions;
    },
    clearDragged: () => {
      interactions.dragged = null;
      interactions = interactions;
    },
    setMaybePeer: (node: Path) => {
      interactions.maybePeer = node;
      interactions = interactions;
    },
    setMaybeChild: (node: Path) => {
      interactions.maybeChild = node;
      interactions = interactions;
    },
    clearMaybePeerAndChild: () => {
      interactions.maybePeer = null;
      interactions.maybeChild = null;
      interactions = interactions;
    },
  });

  let interactions: Interactions = {
    dragged: null,
    ghost: null,
    maybePeer: null,
    maybeChild: null,
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
      <div class="w-40 px-2">Description</div>
    </div>
  </div>

  <div
    id="body"
    class="[&>*:nth-child(even)]:bg-gray-100 [&>*:nth-child(odd)]:bg-gray-200"
  >
    {#each roots as root}
      <Row {graph} {interactions} path={root} />
    {/each}
  </div>
</div>
