<script lang="ts" context="module">
  export type IndexedNode = {
    node: Node;
    offset: number;
  };

  export type DropEffect = "link" | "move" | "none";

  export type Interactions = {
    dragged: IndexedNode | null;
    ghost: IndexedNode | null;
    dropEffect: DropEffect;
    highlighted: Node | null;
  };

  export type TableContext = {
    koso: Koso;
    setDragged: (node: Node, offset: number) => void;
    clearDragged: () => void;
    setDropEffect: (dropEffect: DropEffect) => void;
    setGhost: (node: Node, offset: number) => void;
    clearGhost: () => void;
    setHighlighted: (node: Node) => void;
    clearHighlighted: () => void;
  };
</script>

<script lang="ts">
  import type { Koso } from "$lib/koso";
  import { setContext } from "svelte";
  import { Node, type Graph } from ".";
  import Row from "./row.svelte";

  export let koso: Koso;
  let graph: Graph = {};

  koso.observe(() => {
    graph = koso.toJSON();
  });

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
    koso,
    setDragged: (node: Node, offset: number) => {
      if (
        interactions.dragged &&
        node.equals(interactions.dragged.node) &&
        offset === interactions.dragged.offset
      ) {
        return;
      }
      interactions.dragged = { node, offset };
      interactions = interactions;
    },
    clearDragged: () => {
      interactions.dragged = null;
      interactions.dropEffect = "none";
      interactions = interactions;
    },
    setDropEffect: (dropEffect: DropEffect) => {
      if (interactions.dropEffect === dropEffect) {
        return;
      }
      interactions.dropEffect = dropEffect;
      interactions = interactions;
    },
    setGhost: (node: Node, offset: number) => {
      if (
        interactions.ghost &&
        node.equals(interactions.ghost.node) &&
        offset === interactions.ghost.offset
      ) {
        return;
      }
      interactions.ghost = { node, offset };
      interactions = interactions;
    },
    clearGhost: () => {
      if (interactions.ghost === null) {
        return;
      }
      interactions.ghost = null;
      interactions = interactions;
    },
    setHighlighted: (node: Node) => {
      if (node.equals(interactions.highlighted)) {
        return;
      }
      interactions.highlighted = node;
      interactions = interactions;
    },
    clearHighlighted: () => {
      if (interactions.highlighted === null) {
        return;
      }
      interactions.highlighted = null;
      interactions = interactions;
    },
  });

  let interactions: Interactions = {
    dragged: null,
    dropEffect: "none",
    ghost: null,
    highlighted: null,
  };
</script>

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
      <div class="w-96 border-r px-2">Name</div>
      <div class="w-96 border-r px-2">Reporter</div>
      <div class="w-96 px-2">Assignee</div>
    </div>
  </div>

  <div
    id="body"
    class="[&>*:nth-child(even)]:bg-slate-50 [&>*:nth-child(odd)]:bg-slate-100"
  >
    {#each roots as root, offset}
      <Row {graph} {interactions} isGhost={false} node={root} {offset} />
    {/each}
  </div>
</div>
