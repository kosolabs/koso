<script lang="ts">
  import { user } from "$lib/auth";
  import type { Koso } from "$lib/koso";
  import { Button } from "flowbite-svelte";
  import { List, ListStart, ListTree, Trash, Unlink } from "lucide-svelte";
  import { setContext } from "svelte";
  import { flip } from "svelte/animate";
  import { getOffset, getTask, Node, type Graph } from "../koso";
  import Row from "./row.svelte";
  import { selected } from "./state";
  import { receive, send } from "./transition";

  export let koso: Koso;
  let graph: Graph = koso.toJSON();

  koso.observe(() => {
    graph = koso.toJSON();
  });

  function findRoots(graph: Graph): string[] {
    const allChildren = new Set<string>();
    for (const node of Object.values(graph)) {
      for (const child of node.children) {
        allChildren.add(child);
      }
    }
    const allNodeIds = new Set<string>(Object.keys(graph));
    return Array.from(allNodeIds.difference(allChildren));
  }

  function flatten(node: Node, nodes: Node[]) {
    nodes.push(node);
    for (const child of getTask(graph, node.name).children) {
      flatten(node.concat(child), nodes);
    }
  }

  function toListOfNodes(graph: Graph): Node[] {
    const roots = findRoots(graph);
    const nodes: Node[] = [];
    for (const root of roots) {
      flatten(new Node([root]), nodes);
    }
    return nodes;
  }

  function addRoot() {
    if (!$user) throw new Error("Unauthenticated");
    koso.addRoot($user);
  }

  function addPeer() {
    if (!$selected) return;
    if (!$user) throw new Error("Unauthenticated");
    if ($selected.isRoot()) {
      const newNodeId = koso.addRoot($user);
      $selected = new Node([newNodeId]);
    } else {
      const parent = $selected.parent();
      const newNodeId = koso.insertNode(
        parent.name,
        getOffset(graph, $selected) + 1,
        $user,
      );
      $selected = parent.concat(newNodeId);
    }
  }

  function addChild() {
    if (!$selected) return;
    if (!$user) throw new Error("Unauthenticated");
    const newNodeId = koso.insertNode($selected.name, 0, $user);
    $selected = $selected.concat(newNodeId);
  }

  function unlink() {
    if (!$selected) return;
    koso.removeNode($selected.name, $selected.parent().name);
    $selected = null;
  }

  function remove() {
    if (!$selected) return;
    koso.deleteNode($selected.name);
    $selected = null;
  }

  $: nodes = toListOfNodes(graph);

  setContext<Koso>("koso", koso);
</script>

<div class="my-2 flex gap-2">
  {#if $selected}
    <Button size="xs" on:click={addPeer}>
      <List class="me-2 w-4" />Add Peer
    </Button>
    <Button size="xs" on:click={addChild}>
      <ListTree class="me-2 w-4" />Add Child
    </Button>
    {#if $selected.isRoot()}
      <Button size="xs" on:click={remove}>
        <Trash class="me-2 w-4" />Delete
      </Button>
    {:else}
      <Button size="xs" on:click={unlink}>
        <Unlink class="me-2 w-4" />Unlink
      </Button>
    {/if}
  {:else}
    <Button size="xs" on:click={addRoot}>
      <ListStart class="me-2 w-4" />Add Root
    </Button>
  {/if}
</div>

<div class="rounded-t border">
  <div id="header" class="border-b text-xs font-bold uppercase">
    <div class="my-1 flex items-center p-2">
      <div class="min-w-48 whitespace-nowrap border-r">
        <div class="flex items-center">
          <div class="w-5"></div>
          <div class="w-5"></div>
          <div>ID</div>
        </div>
      </div>
      <div class="w-96 whitespace-nowrap border-r px-2">Name</div>
      <div class="w-96 whitespace-nowrap border-r px-2">Reporter</div>
      <div class="w-96 whitespace-nowrap px-2">Assignee</div>
    </div>
  </div>

  <div id="body" class="[&>*:nth-child(even)]:bg-slate-50">
    {#each nodes as node (node.id)}
      <div
        in:receive={{ key: node.id }}
        out:send={{ key: node.id }}
        animate:flip={{ duration: 250 }}
      >
        <Row {graph} isGhost={false} {node} />
      </div>
    {/each}
  </div>
</div>
