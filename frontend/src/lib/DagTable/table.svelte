<script lang="ts" context="module">
  export type Interactions = {
    dragged: Path | null;
    maybePeer: Path | null;
    maybeChild: Path | null;
  };
</script>

<script lang="ts">
  import type { Graph, Path } from ".";
  import Row from "./row.svelte";

  export let graph: Graph;
  export let root: Path;

  let interactions: Interactions = {
    dragged: null,
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
    <Row bind:graph bind:interactions path={root} />
  </div>
</div>
