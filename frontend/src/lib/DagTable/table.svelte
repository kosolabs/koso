<script lang="ts" context="module">
  export type Interactions = {
    dragged: Path | null;
    ghost: { parentId: string; childId: string; offset: number } | null;
    maybePeer: Path | null;
    maybeChild: Path | null;
  };

  export type TableContext = {
    addChild: (parentId: string, childId: string, offset: number) => void;
    removeChild: (child: Path) => void;
    setDragged: (node: Path) => void;
    clearDragged: () => void;
    setMaybePeer: (node: Path) => void;
    setMaybeChild: (node: Path) => void;
    clearMaybePeerAndChild: () => void;
  };
</script>

<script lang="ts">
  import { setContext } from "svelte";

  import type { Graph, Path } from ".";
  import Row from "./row.svelte";

  export let graph: Graph;
  export let root: Path;

  setContext<TableContext>(graph, {
    addChild: (parentId: string, childId: string, offset: number) => {
      const parent = graph[parentId]!;
      parent.children.splice(offset, 0, childId);
      graph = graph;
    },
    removeChild: (child: Path) => {
      const parent = graph[child.parent().name]!;
      parent.children.splice(parent.children.indexOf(child.name), 1);
      graph = graph;
    },
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
    <Row {graph} {interactions} path={root} />
  </div>
</div>
