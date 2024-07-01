import { default as Root, default as Row } from "./table.svelte";

export class Path {
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

  parent(): Path {
    return new Path(this.path.slice(0, -1));
  }

  concat(nodeId: string) {
    return new Path(this.path.concat(nodeId));
  }

  equals(other: Path | null): boolean {
    if (other === null) {
      return false;
    }
    return this.id === other.id;
  }
}

export type Node = {
  id: string;
  title: string;
  children: string[];
};

export type Graph = {
  [key: string]: Node;
};

export { Root as DagTable, Row as DagTableRow, Root };
