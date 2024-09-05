import * as decoding from "lib0/decoding";
import * as encoding from "lib0/encoding";
import {
  derived,
  readable,
  writable,
  type Readable,
  type Unsubscriber,
  type Writable,
} from "svelte/store";
import { v4 as uuidv4 } from "uuid";
import { IndexeddbPersistence } from "y-indexeddb";
import * as Y from "yjs";
import type { User } from "./auth";
import { storable } from "./stores";

const MSG_SYNC = 0;
// const MSG_AWARENESS = 1;
// const MSG_AUTH = 2;
// const MSG_QUERY_AWARENESS = 3;

const MSG_SYNC_REQUEST = 0;
const MSG_SYNC_RESPONSE = 1;
const MSG_SYNC_UPDATE = 2;

export class Node {
  #koso: Koso;
  path: string[];
  offset: number;

  static get separator() {
    return "/";
  }

  static id(path: string[]) {
    return path.join(Node.separator) || "root";
  }

  get id() {
    return Node.id(this.path);
  }

  static name(path: string[]) {
    return path.at(-1) ?? "root";
  }

  get name(): string {
    return Node.name(this.path);
  }

  get length(): number {
    return this.path.length;
  }

  static concat(path: string[], child: string): string {
    return Node.id(path.concat(child));
  }

  constructor(koso: Koso, path: string[], offset: number) {
    this.#koso = koso;
    this.path = path;
    this.offset = offset;
  }

  parent(): Node {
    return this.#koso.getParent(this);
  }

  child(name: string): Node {
    return this.#koso.getChild(this, name);
  }

  equals(other: Node | null): boolean {
    if (other === null) {
      return false;
    }
    return this.id === other.id;
  }
}

export type Status = "Not Started" | "In Progress" | "Done";

export type Task = {
  id: string;
  num: string;
  name: string;
  children: string[];
  assignee: string | null;
  reporter: string | null;
  status: Status | null;
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type YEvent = Y.YEvent<any>;

export type Graph = { [id: string]: Task };
export type Parents = { [id: string]: string[] };

export class Koso {
  yDoc: Y.Doc;
  yGraph: Y.Map<Y.Map<Y.Array<string> | string | null>>;
  yIndexedDb: IndexeddbPersistence;
  clientMessageHandler: (message: Uint8Array) => void;

  events: Readable<YEvent[]>;
  selectedId: Writable<string | null>;
  highlighted: Writable<string | null>;
  dropEffect: Writable<"link" | "move" | "none">;
  draggedId: Writable<string | null>;
  expanded: Writable<Set<string>>;
  parents: Readable<Parents>;
  nodesAndIds: Readable<[string[], { [id: string]: Node }]>;
  nodeIds: string[] = [];
  nodes: { [id: string]: Node } = {};
  unsubscribe: Unsubscriber;

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

    this.events = readable<YEvent[]>([], (set) => {
      const observer = (events: YEvent[]) => set(events);
      this.observe(observer);
      return () => this.unobserve(observer);
    });

    this.selectedId = writable<string | null>(null);
    this.highlighted = writable<string | null>(null);
    this.dropEffect = writable<"link" | "move" | "none">("none");
    this.draggedId = writable<string | null>(null);

    const expandedLocalStorageKey = `expanded-nodes-${projectId}`;
    this.expanded = storable<Set<string>>(
      expandedLocalStorageKey,
      new Set(),
      (json: string) => new Set<string>(JSON.parse(json)),
      (value) => JSON.stringify(Array.from(value)),
    );

    this.nodesAndIds = derived([this.expanded, this.events], ([expanded]) =>
      this.#flatten(new Node(this, [], 0), [], {}, expanded),
    );
    this.parents = derived([this.events], () => this.#toParents());

    this.unsubscribe = this.nodesAndIds.subscribe(($nodesAndIds) => {
      [this.nodeIds, this.nodes] = $nodesAndIds;
    });
  }

  observe(f: (arg0: YEvent[], arg1: Y.Transaction) => void) {
    this.yGraph.observeDeep(f);
  }

  unobserve(f: (arg0: YEvent[], arg1: Y.Transaction) => void) {
    this.yGraph.unobserveDeep(f);
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
      } else if (syncType === MSG_SYNC_RESPONSE) {
        const message = decoding.readVarUint8Array(decoder);
        Y.applyUpdateV2(this.yDoc, message);
        if (this.yGraph.size === 0) {
          this.#upsertRoot([]);
        }
      } else if (syncType === MSG_SYNC_UPDATE) {
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

  toJSON() {
    return this.yGraph.toJSON();
  }

  #flatten(
    node: Node,
    nodeIds: string[],
    nodes: { [id: string]: Node },
    expanded: Set<string>,
  ): [string[], { [id: string]: Node }] {
    const task = this.yGraph.get(node.name);
    if (task) {
      nodeIds.push(node.id);
      nodes[node.id] = node;
      if (node.length < 1 || expanded.has(node.id)) {
        (task.get("children") as Y.Array<string>).forEach((name, offset) => {
          const child = new Node(this, node.path.concat(name), offset);
          this.#flatten(child, nodeIds, nodes, expanded);
        });
      }
    }
    return [nodeIds, nodes];
  }

  #toParents(): Parents {
    const parents: Parents = {};
    for (const [parentId, task] of this.yGraph.entries()) {
      for (const childId of task.get("children") as Y.Array<string>) {
        if (!(childId in parents)) {
          parents[childId] = [];
        }
        parents[childId].push(parentId);
      }
    }
    return parents;
  }

  getNode(key: string | number): Node {
    if (typeof key === "string") {
      if (!this.nodes[key])
        throw new Error(`Node ID ${key} not found in nodes`);
      return this.nodes[key];
    }
    if (typeof key === "number") {
      return this.nodes[this.nodeIds[key]];
    }
    throw new Error(`Node ID ${key} not found in nodes`);
  }

  getParent(node: Node): Node {
    if (node.path.length === 0)
      throw new Error("Cannot get parent of root node");
    return this.getNode(Node.id(node.path.slice(0, -1)));
  }

  getChild(node: Node, childName: string): Node {
    return this.getNode(Node.id(node.path.concat(childName)));
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

  getOrphanedTaskIds() {
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

    return Array.from(allTaskIds.difference(allChildTaskIds));
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

  #upsertRoot(children: string[]) {
    this.#upsert({
      id: "root",
      num: "0",
      name: "Root",
      children: children,
      reporter: null,
      assignee: null,
      status: null,
    });
  }

  #upsert(task: Task) {
    this.yGraph.set(
      task.id,
      new Y.Map<Y.Array<string> | string | null>([
        ["id", task.id],
        ["num", task.num],
        ["name", task.name],
        ["children", Y.Array.from(task.children)],
        ["reporter", task.reporter],
        ["assignee", task.assignee],
        ["status", task.status],
      ]),
    );
  }

  linkNode(node: Node, parent: string, offset: number) {
    this.yDoc.transact(() => {
      const yParent = this.yGraph.get(parent);
      if (!yParent) throw new Error(`Task ${parent} is not in the graph`);
      const yChildren = yParent.get("children") as Y.Array<string>;
      yChildren.insert(offset, [node.name]);
    });
  }

  #unlinkNode(node: Node) {
    const nodeId = node.name;
    const parentId = this.getParent(node).name;
    const yParent = this.yGraph.get(parentId);
    if (!yParent) throw new Error(`Task ${parentId} is not in the graph`);
    const yParentsChildren = yParent.get("children") as Y.Array<string>;
    const yParentsChildrenArr = yParentsChildren.toArray();
    yParentsChildren.delete(yParentsChildrenArr.indexOf(nodeId));

    const yNode = this.yGraph.get(nodeId);
    if (!yNode) throw new Error(`Task ${nodeId} is not in the graph`);
    const yChildren = yNode.get("children") as Y.Array<string>;
    for (const child of yChildren) {
      if (!yParentsChildrenArr.includes(child)) {
        yParentsChildren.push([child]);
      }
    }
  }

  unlinkNode(node: Node) {
    this.yDoc.transact(() => {
      this.#unlinkNode(node);
    });
  }

  #deleteNode(node: Node) {
    this.yGraph.delete(node.name);
  }

  deleteNode(node: Node) {
    this.yDoc.transact(() => {
      this.#unlinkNode(node);
      this.#deleteNode(node);
    });
  }

  moveNode(node: Node, destParentId: string, destOffset: number) {
    this.yDoc.transact(() => {
      const srcParentId = node.parent().name;
      const ySrcParent = this.yGraph.get(srcParentId);
      if (!ySrcParent)
        throw new Error(`Task ${srcParentId} is not in the graph`);
      const ySrcChildren = ySrcParent.get("children") as Y.Array<string>;
      ySrcChildren.delete(node.offset);

      const yDestParent = this.yGraph.get(destParentId);
      if (!yDestParent)
        throw new Error(`Task ${destParentId} is not in the graph`);
      const yDestChildren = yDestParent.get("children") as Y.Array<string>;
      if (srcParentId === destParentId && node.offset < destOffset) {
        destOffset -= 1;
      }
      yDestChildren.insert(destOffset, [node.name]);
    });
  }

  insertNode(parent: Node, offset: number, name: string, user: User): string {
    const taskId = this.newId();
    this.yDoc.transact(() => {
      this.#upsert({
        id: taskId,
        num: this.newNum(),
        name: name,
        children: [],
        reporter: user.email,
        assignee: null,
        status: null,
      });
      const yParent = this.yGraph.get(parent.name)!;
      const yChildren = yParent.get("children") as Y.Array<string>;
      yChildren.insert(offset, [taskId]);
    });
    return Node.id(parent.path.concat(taskId));
  }

  setTaskName(taskId: string, newName: string) {
    this.yDoc.transact(() => {
      const yNode = this.yGraph.get(taskId);
      if (!yNode) throw new Error(`Task ${taskId} is not in the graph`);
      if (yNode.get("name") !== newName) {
        yNode.set("name", newName);
      }
    });
  }

  setAssignee(taskId: string, assignee: User | null) {
    this.yDoc.transact(() => {
      const yNode = this.yGraph.get(taskId);
      if (!yNode) throw new Error(`Task ${taskId} is not in the graph`);
      if (assignee === null && yNode.get("assignee") !== null) {
        yNode.set("assignee", null);
      } else if (assignee && assignee.email !== yNode.get("assignee")) {
        yNode.set("assignee", assignee.email);
      }
    });
  }

  setReporter(taskId: string, reporter: User | null) {
    this.yDoc.transact(() => {
      const yNode = this.yGraph.get(taskId);
      if (!yNode) throw new Error(`Task ${taskId} is not in the graph`);
      if (reporter === null && yNode.get("reporter") !== null) {
        yNode.set("reporter", null);
      } else if (reporter && reporter.email !== yNode.get("reporter")) {
        yNode.set("reporter", reporter.email);
      }
    });
  }

  setTaskStatus(taskId: string, status: Status | null) {
    this.yDoc.transact(() => {
      const yNode = this.yGraph.get(taskId);
      if (!yNode) throw new Error(`Task ${taskId} is not in the graph`);
      if (yNode.get("status") !== status) {
        yNode.set("status", status);
      }
    });
  }
}
