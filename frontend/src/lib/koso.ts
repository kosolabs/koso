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
  depth: number;
  _parentNodeId: string | null;

  constructor(id: string, parentNodeId: string | null, depth: number) {
    this.id = id;
    this.depth = depth;
    this._parentNodeId = parentNodeId;
  }

  taskId(): string {
    return this.id.split("|")[0];
  }

  parentNodeId(): string {
    if (!this._parentNodeId) {
      throw new Error("Root has no parent");
    }
    return this._parentNodeId;
  }

  parentTaskId(): string {
    return this.parentNodeId().split("|")[0];
  }

  isRoot(): boolean {
    return !this._parentNodeId;
  }

  // equals(other: Node | null): boolean {
  //   if (other === null) {
  //     return false;
  //   }
  //   return this.id === other.id;
  // }

  new_child_node(id: string): Node {
    return new Node(id, this.id, this.depth + 1);
  }
  new_peer_node(id: string): Node {
    return new Node(id, this.parentNodeId(), this.depth);
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

    const idMapping: { [id: string]: string } = {};
    for (const task of this.yGraph.values()) {
      const num = task.get("num") as string;
      if (num) {
        idMapping[num] = task.get("id") as string;
      } else {
        idMapping[task.get("id") as string] = this.newId();
      }
    }
    this.yDoc.transact(() => {
      for (const task of this.yGraph.values()) {
        const taskId = task.get("id") as string;
        if (task.get("num")) {
          console.log(`Converting tasking ${taskId}...`);
          this.yGraph.delete(taskId);
          const newTaskId = idMapping[taskId];
          if (!newTaskId) {
            throw Error("Id mapping missing");
          }
          task.set("id", newTaskId);

          const childNodeIds = [];
          for (const childTaskId of task.get("children") as Y.Array<string>) {
            const newChildTaskId = idMapping[childTaskId];
            if (!newChildTaskId) {
              throw Error("Id mapping missing");
            }
            childNodeIds.push(this.newChildNodeId(childTaskId));
          }
          task.set("children", Y.Array.from(childNodeIds));
          console.log(
            `Converted task ${taskId}, new id is ${newTaskId}. Task: ${task.toJSON()}`,
          );
          this.yGraph.set(newTaskId, task);
        }
      }
    });
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

  getRootNodeIds(): Set<string> {
    const allChildTaskIds = new Set<string>();
    for (const task of this.yGraph.values()) {
      for (const childNodeId of task.get("children") as Y.Array<string>) {
        allChildTaskIds.add(this.splitChildNodeId(childNodeId)[0]);
      }
    }
    const rootNodeIds = new Set<string>();
    for (const task of this.yGraph.values()) {
      const taskId = task.get("id") as string;
      if (!allChildTaskIds.has(taskId)) {
        rootNodeIds.add(taskId);
      }
    }
    return rootNodeIds;
  }

  getTask(taskId: string): Task {
    const yTask = this.yGraph.get(taskId);
    if (!yTask) throw new Error(`Task ID ${taskId} not found in yGraph`);
    return yTask.toJSON() as Task;
  }

  getChildrenTaskIds(taskId: string): string[] {
    return this.getChildrenNodeIds(taskId).map(
      (c) => this.splitChildNodeId(c)[0],
    );
  }

  getChildrenNodeIds(taskId: string): string[] {
    const yTask = this.yGraph.get(taskId);
    if (!yTask) throw new Error(`Task ID ${taskId} not found in yGraph`);
    const yChildren = yTask.get("children") as Y.Array<string>;
    return yChildren.toArray();
  }

  newChildNodeId(childTaskId: string): string {
    return childTaskId + "|" + this.newId();
  }

  splitChildNodeId(nodeId: string): [string, string] {
    const parts = nodeId.split("|");
    if (parts.length != 2) {
      throw new Error("Expected two parts from node id: " + nodeId);
    }
    return [parts[0], parts[1]];
  }

  getOffset(node: Node): number {
    if (node.isRoot()) return 0;
    const task = this.getTask(node.parentTaskId());
    return task.children.indexOf(node.id);
  }

  #flatten(node: Node, nodes: Node[]) {
    nodes.push(node);
    for (const childNodeId of this.getChildrenNodeIds(node.taskId())) {
      this.#flatten(node.new_child_node(childNodeId), nodes);
    }
  }

  toNodes(): Node[] {
    const roots = this.getRootNodeIds();
    const nodes: Node[] = [];
    for (const root of roots) {
      this.#flatten(new Node(root, null, 0), nodes);
    }
    return nodes;
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

  newId(): string {
    return uuidv4();
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
    const taskId = this.newId();
    this.yDoc.transact(() => {
      this.upsert({
        id: taskId,
        num: this.newNum(),
        name: "Untitled",
        children: [],
        reporter: user.email,
        assignee: null,
      });
    });
    return taskId;
  }

  addNode(taskId: string, parentTaskId: string, offset: number) {
    const childNodeId = this.newChildNodeId(taskId);
    this.yDoc.transact(() => {
      const yParent = this.yGraph.get(parentTaskId);
      if (!yParent) throw new Error(`Task ${parentTaskId} is not in the graph`);
      const yChildren = yParent.get("children") as Y.Array<string>;
      yChildren.insert(offset, [childNodeId]);
    });
  }

  removeNode(nodeId: string, parentTaskId: string) {
    this.yDoc.transact(() => {
      const yParent = this.yGraph.get(parentTaskId);
      if (!yParent) throw new Error(`Task ${parentTaskId} is not in the graph`);
      const yChildren = yParent.get("children") as Y.Array<string>;
      yChildren.delete(yChildren.toArray().indexOf(nodeId));
    });
  }

  deleteTask(taskId: string) {
    this.yDoc.transact(() => {
      this.yGraph.delete(taskId);
    });
  }

  moveNode(
    nodeId: string,
    srcParentTaskId: string,
    srcOffset: number,
    destParentTaskId: string,
    destOffset: number,
  ) {
    this.yDoc.transact(() => {
      const ySrcParent = this.yGraph.get(srcParentTaskId);
      if (!ySrcParent)
        throw new Error(`Task ${srcParentTaskId} is not in the graph`);
      const ySrcChildren = ySrcParent.get("children") as Y.Array<string>;
      ySrcChildren.delete(srcOffset);

      const yDestParent = this.yGraph.get(destParentTaskId);
      if (!yDestParent)
        throw new Error(`Task ${destParentTaskId} is not in the graph`);
      const yDestChildren = yDestParent.get("children") as Y.Array<string>;
      if (destParentTaskId === destParentTaskId && srcOffset < destOffset) {
        destOffset -= 1;
      }
      yDestChildren.insert(destOffset, [nodeId]);
    });
  }

  insertNode(parentTaskId: string, offset: number, user: User): string {
    const taskId = this.newId();
    const childNodeId = this.newChildNodeId(taskId);
    this.yDoc.transact(() => {
      this.upsert({
        id: taskId,
        num: this.newNum(),
        name: "Untitled",
        children: [],
        reporter: user.email,
        assignee: null,
      });
      const yParent = this.yGraph.get(parentTaskId)!;
      if (!yParent) throw new Error(`Task ${parentTaskId} is not in the graph`);
      const yChildren = yParent.get("children") as Y.Array<string>;
      yChildren.insert(offset, [childNodeId]);
    });
    return childNodeId;
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
