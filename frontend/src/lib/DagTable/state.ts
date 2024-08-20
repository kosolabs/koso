import { page } from "$app/stores";
import { Node, type Task } from "$lib/koso";
import { storedWritable } from "$lib/stores";
import { derived, writable } from "svelte/store";

type Graph = { [id: string]: Task };
type Parents = { [id: string]: string[] };

export const project = derived(page, (page) => page.params?.projectId);
export const graph = writable<Graph>({});

export const expanded = storedWritable<Set<string>>(
  "expanded-nodes-",
  project,
  new Set(),
  (json: string) => new Set<string>(JSON.parse(json)),
  (value) => JSON.stringify(Array.from(value)),
);

export const selected = writable<Node | null>(null);
export const highlighted = writable<Node | null>(null);
export const dropEffect = writable<"link" | "move" | "none">("none");
export const dragged = writable<Node | null>(null);

function flatten(
  node: Node,
  nodes: Node[],
  graph: Graph,
  expanded: Set<string>,
): Node[] {
  if (node.name in graph && (node.length < 1 || expanded.has(node.id))) {
    for (const childName of graph[node.name].children) {
      const child = node.concat(childName);
      nodes.push(child);
      flatten(child, nodes, graph, expanded);
    }
  }
  return nodes;
}

export const nodes = derived([graph, expanded], ([graph, expanded]) =>
  flatten(new Node([]), [], graph, expanded),
);

export const parents = derived(graph, (graph) => {
  const parents: Parents = {};
  for (const [parentId, task] of Object.entries(graph)) {
    for (const childId of task.children) {
      if (!(childId in parents)) {
        parents[childId] = [];
      }
      parents[childId].push(parentId);
    }
  }
  return parents;
});

export const isExpanded = derived(
  expanded,
  ($expanded) => (nodeId: string) => $expanded.has(nodeId),
);

export const expand = derived(expanded, ($expanded) => (nodeId: string) => {
  $expanded.add(nodeId);
  expanded.set($expanded);
});

export const collapse = derived(expanded, ($expanded) => (nodeId: string) => {
  $expanded.delete(nodeId);
  expanded.set($expanded);
});
