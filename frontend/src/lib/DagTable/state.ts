import { derived, writable } from "svelte/store";
import { Node, type Task } from "../koso";

type Graph = { [id: string]: Task };
type Parents = { [id: string]: string[] };

export const graph = writable<Graph>({});
export const expanded = writable<Set<string>>(new Set());

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

export const nodes = derived([graph, expanded], ([graph, expanded]) => {
  const nodes = flatten(new Node([]), [], graph, expanded);
  console.log(expanded);
  return nodes;
});

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
