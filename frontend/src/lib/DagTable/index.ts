import { default as Root, default as Row } from "./table.svelte";

export class Node {
  id: string;
  name: string;
  length: number;
  path: string[];

  constructor(path: string[]) {
    this.path = path;
    this.id = this.path.join("-");
    this.name = this.path.at(-1)!;
    this.length = this.path.length;
  }

  parent(): Node {
    return new Node(this.path.slice(0, -1));
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

export type Task = {
  id: string;
  name: string;
  children: string[];
};

export type Graph = {
  [key: string]: Task;
};

export { Root as DagTable, Row as DagTableRow, Root };
