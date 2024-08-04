import * as decoding from "lib0/decoding";
import * as encoding from "lib0/encoding";
import { IndexeddbPersistence } from "y-indexeddb";
import * as Y from "yjs";
import type { User } from "./auth";
import { v4 as uuidv4 } from "uuid";

const MSG_SYNC = 0;
// const MSG_AWARENESS = 1;
// const MSG_AUTH = 2;
// const MSG_QUERY_AWARENESS = 3;

const MSG_SYNC_REQUEST = 0;
const MSG_SYNC_RESPONSE = 1;
const MSG_SYNC_UPDATE = 2;

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
  num: string;
  name: string;
  children: string[];
  assignee: string | null;
  reporter: string;
};

export class Koso {
  yDoc: Y.Doc;
  yGraph: Y.Map<Y.Map<string | Y.Array<string>>>;
  yIndexedDb: IndexeddbPersistence;
  clientMessageHandler: (message: Uint8Array) => void;

  constructor(projectId: string, yDoc: Y.Doc) {
    this.yDoc = yDoc;
    this.yGraph = yDoc.getMap("graph");
    this.yIndexedDb = new IndexeddbPersistence(`koso-${projectId}`, this.yDoc);
    this.clientMessageHandler = () => {
      console.warn("Client message handler was invoked but was not set");
    };

    this.yDoc.on(
      "updateV2",
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (message: Uint8Array, arg1: any, arg2: Y.Doc, txn: Y.Transaction) => {
        if (txn.local) {
          const encoder = encoding.createEncoder();
          encoding.writeVarUint(encoder, MSG_SYNC);
          encoding.writeVarUint(encoder, MSG_SYNC_UPDATE);
          encoding.writeVarUint8Array(encoder, message);
          this.clientMessageHandler(encoding.toUint8Array(encoder));
        }
      },
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  observe(f: (arg0: Array<Y.YEvent<any>>, arg1: Y.Transaction) => void) {
    this.yGraph.observeDeep(f);
  }

  handleServerMessage(message: Uint8Array) {
    const decoder = decoding.createDecoder(message);
    const messageType = decoding.readVarUint(decoder);
    if (messageType === MSG_SYNC) {
      const syncType = decoding.readVarUint(decoder);

      if (syncType === MSG_SYNC_REQUEST) {
        const encoder = encoding.createEncoder();
        const encodedStateVector = decoding.readVarUint8Array(decoder);
        encoding.writeVarUint(encoder, MSG_SYNC);
        encoding.writeVarUint(encoder, MSG_SYNC_RESPONSE);
        encoding.writeVarUint8Array(
          encoder,
          Y.encodeStateAsUpdateV2(this.yDoc, encodedStateVector),
        );
        this.clientMessageHandler(encoding.toUint8Array(encoder));
      } else if (
        syncType === MSG_SYNC_RESPONSE ||
        syncType === MSG_SYNC_UPDATE
      ) {
        const message = decoding.readVarUint8Array(decoder);
        Y.applyUpdateV2(this.yDoc, message);
        this.convertToTaskNum();
      } else {
        throw new Error(`Unknown sync type: ${syncType}`);
      }
    } else {
      throw new Error(
        `Expected message type to be Sync (0) but was: ${messageType}`,
      );
    }
  }

  handleClientMessage(f: (message: Uint8Array) => void) {
    this.clientMessageHandler = f;

    const encoder = encoding.createEncoder();
    encoding.writeVarUint(encoder, MSG_SYNC);
    encoding.writeVarUint(encoder, MSG_SYNC_REQUEST);
    const sv = Y.encodeStateVector(this.yDoc);
    encoding.writeVarUint8Array(encoder, sv);
    this.clientMessageHandler(encoding.toUint8Array(encoder));
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
    return uuidv4();
  }

  newNum(): string {
    let max = 0;
    for (const task of this.yGraph.values()) {
      const currNum = task.get("num") as string;
      const curr = parseInt(currNum);
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
          ["num", task.num],
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
        num: this.newNum(),
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
        num: this.newNum(),
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

  convertToTaskNum() {
    console.log("Working on ygraph", this.yGraph.toJSON());
    const idMapping: { [id: string]: string } = {};
    for (const task of this.yGraph.values()) {
      const num = task.get("num") as string;
      if (num) {
        idMapping[num] = task.get("id") as string;
      } else {
        idMapping[task.get("id") as string] = this.newId();
      }
    }
    console.log(`Going to convert with mappings`, idMapping);
    this.yDoc.transact(() => {
      for (const task of this.yGraph.values()) {
        if (task.get("num")) {
          console.log(`Task already converted`, task.toJSON());
          continue;
        }

        const taskNum = task.get("id") as string;
        console.log(`Converting task ${taskNum}...`, task.toJSON());
        const newTaskId = idMapping[taskNum];
        if (!newTaskId) {
          throw Error("Id mapping missing");
        }

        const childNodeIds = [];
        for (const childTaskId of task.get("children") as Y.Array<string>) {
          const newChildTaskId = idMapping[childTaskId];
          if (!newChildTaskId) throw Error("Id mapping missing");
          childNodeIds.push(newChildTaskId);
        }

        const newTask = new Y.Map<string | Y.Array<string>>([
          ["id", newTaskId],
          ["num", taskNum],
          ["name", task.get("name")],
          ["children", Y.Array.from(childNodeIds)],
          ["reporter", task.get("reporter")],
          ["assignee", task.get("assignee")],
        ]);
        this.yGraph.delete(taskNum);
        this.yGraph.set(newTaskId, newTask);

        console.log(
          `Converted task ${taskNum}, new id is ${newTaskId}.`,
          newTask.toJSON(),
        );
      }
    });
    console.log("Finished converting", this.yGraph.toJSON());
  }
}
