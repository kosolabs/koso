import * as protocol from "$lib/protocol";
import { IndexeddbPersistence } from "y-indexeddb";
import * as Y from "yjs";
import type { User } from "./auth";

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

export type Task = {
  id: string;
  name: string;
  children: string[];
  assignee: string | null;
  reporter: string;
};

export class Koso {
  yDoc: Y.Doc;
  yGraph: Y.Map<Y.Map<string | Y.Array<string>>>;
  yIndexedDb: IndexeddbPersistence;

  constructor(projectId: string, yDoc: Y.Doc) {
    this.yDoc = yDoc;
    this.yGraph = yDoc.getMap("graph");
    this.yIndexedDb = new IndexeddbPersistence(`koso-${projectId}`, this.yDoc);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  observe(f: (arg0: Array<Y.YEvent<any>>, arg1: Y.Transaction) => void) {
    this.yGraph.observeDeep(f);
  }

  createSyncRequest(): Uint8Array {
    return protocol.syncRequest(this.yDoc);
  }

  onLocalUpdate(
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    f: (arg0: Uint8Array, arg1: any, arg2: Y.Doc, arg3: Y.Transaction) => void,
  ) {
    this.yDoc.on(
      "update",
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (update: Uint8Array, arg1: any, arg2: Y.Doc, txn: Y.Transaction) => {
        if (txn.local) {
          f(update, arg1, arg2, txn);
        }
      },
    );
  }

  update(data: Uint8Array) {
    Y.applyUpdateV2(this.yDoc, data);
  }

  getRoots(): Set<string> {
    const allChildTaskIds = new Set<string>();
    for (const task of this.yGraph.values()) {
      for (const childTaskId of task.get("children") as Y.Array<string>) {
        allChildTaskIds.add(childTaskId);
      }
    }
    const allTaskIds = new Set<string>();
    for (const taskId of this.yGraph.keys()) {
      allTaskIds.add(taskId);
    }
    return allTaskIds.difference(allChildTaskIds);
  }

  getTask(taskId: string): Task {
    const yTask = this.yGraph.get(taskId);
    if (!yTask) throw new Error(`Task ID ${taskId} not found in yGraph`);
    return yTask.toJSON() as Task;
  }

  getChildren(taskId: string): string[] {
    const yTask = this.yGraph.get(taskId);
    if (!yTask) throw new Error(`Task ID ${taskId} not found in yGraph`);
    const yChildren = yTask.get("children") as Y.Array<string>;
    return yChildren.toArray();
  }

  getOffset(node: Node): number {
    if (node.isRoot()) return 0;
    const task = this.getTask(node.parent().name);
    return task.children.indexOf(node.name);
  }

  #flatten(node: Node, nodes: Node[]) {
    nodes.push(node);
    for (const child of this.getChildren(node.name)) {
      this.#flatten(node.concat(child), nodes);
    }
  }

  toNodes(): Node[] {
    const roots = this.getRoots();
    const nodes: Node[] = [];
    for (const root of roots) {
      this.#flatten(new Node([root]), nodes);
    }
    return nodes;
  }

  newId(): string {
    let max = 0;
    for (const currId of this.yGraph.keys()) {
      const curr = parseInt(currId);
      if (curr > max) {
        max = curr;
      }
    }
    return `${max + 1}`;
  }

  upsert(task: Task) {
    this.yDoc.transact(() => {
      this.yGraph.set(
        task.id,
        new Y.Map<string | Y.Array<string>>([
          ["id", task.id],
          ["name", task.name],
          ["children", Y.Array.from(task.children)],
          ["reporter", task.reporter],
          ["assignee", task.assignee],
        ]),
      );
    });
  }

  addRoot(user: User): string {
    const nodeId = this.newId();
    this.yDoc.transact(() => {
      this.upsert({
        id: nodeId,
        name: "Untitled",
        children: [],
        reporter: user.email,
        assignee: null,
      });
    });
    return nodeId;
  }

  addNode(nodeId: string, parentId: string, offset: number) {
    this.yDoc.transact(() => {
      const yParent = this.yGraph.get(parentId);
      if (!yParent) throw new Error(`Task ${parentId} is not in the graph`);
      const yChildren = yParent.get("children") as Y.Array<string>;
      yChildren.insert(offset, [nodeId]);
    });
  }

  removeNode(nodeId: string, parentId: string) {
    this.yDoc.transact(() => {
      const yParent = this.yGraph.get(parentId);
      if (!yParent) throw new Error(`Task ${parentId} is not in the graph`);
      const yChildren = yParent.get("children") as Y.Array<string>;
      yChildren.delete(yChildren.toArray().indexOf(nodeId));
    });
  }

  deleteNode(nodeId: string) {
    this.yDoc.transact(() => {
      this.yGraph.delete(nodeId);
    });
  }

  moveNode(
    nodeId: string,
    srcParentId: string,
    srcOffset: number,
    destParentId: string,
    destOffset: number,
  ) {
    this.yDoc.transact(() => {
      const ySrcParent = this.yGraph.get(srcParentId);
      if (!ySrcParent)
        throw new Error(`Task ${srcParentId} is not in the graph`);
      const ySrcChildren = ySrcParent.get("children") as Y.Array<string>;
      ySrcChildren.delete(srcOffset);

      const yDestParent = this.yGraph.get(destParentId);
      if (!yDestParent)
        throw new Error(`Task ${destParentId} is not in the graph`);
      const yDestChildren = yDestParent.get("children") as Y.Array<string>;
      if (srcParentId === destParentId && srcOffset < destOffset) {
        destOffset -= 1;
      }
      yDestChildren.insert(destOffset, [nodeId]);
    });
  }

  insertNode(parentId: string, offset: number, user: User): string {
    const nodeId = this.newId();
    this.yDoc.transact(() => {
      this.upsert({
        id: nodeId,
        name: "Untitled",
        children: [],
        reporter: user.email,
        assignee: null,
      });
      const yParent = this.yGraph.get(parentId)!;
      const yChildren = yParent.get("children") as Y.Array<string>;
      yChildren.insert(offset, [nodeId]);
    });
    return nodeId;
  }

  editTaskName(taskId: string, newName: string) {
    this.yDoc.transact(() => {
      const yNode = this.yGraph.get(taskId);
      if (!yNode) throw new Error(`Task ${taskId} is not in the graph`);
      if (yNode.get("name") !== newName) {
        yNode.set("name", newName);
      }
    });
  }
}
