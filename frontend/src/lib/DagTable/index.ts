import { default as DagTable } from "./table.svelte";

export class Node {
  id: string;
  name: string;
  length: number;
  path: string[];

  constructor(path: string[]) {
    this.path = path;
    this.id = this.path.join("-");
    const maybeName = this.path.at(-1);
    if (!maybeName) throw new Error("path should not be empty");
    this.name = maybeName;
    this.length = this.path.length;
  }

  parent(): Node {
    if (this.isRoot()) throw new Error("Cannot get parent of root node");
    return new Node(this.path.slice(0, -1));
  }

  isRoot(): boolean {
    return this.path.length === 1;
  }

  concat(nodeId: string) {
    return new Node(this.path.concat(nodeId));
  }

  equals(other: Node | null): boolean {
    if (other === null) {
      return false;
    }
    return this.id === other.id;
  }
}

export function getTask(graph: Graph, id: string): Task {
  const task = graph[id];
  if (!task) {
    throw new Error(`Task ${id} doesn't exist`);
  }
  return task;
}

export type Task = {
  id: string;
  name: string;
  children: string[];
  assignee: string | null;
  reporter: string;
};

export type Graph = {
  [key: string]: Task;
};

export { DagTable };
