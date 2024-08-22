import * as decoding from "lib0/decoding";
import * as encoding from "lib0/encoding";
import {
  derived,
  readable,
  writable,
  type Readable,
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
  id: string;
  name: string;
  length: number;
  path: string[];

  static get separator() {
    return "/";
  }

  constructor(path: string[]) {
    this.path = path;
    const maybeName = this.path.at(-1);
    if (!maybeName) {
      this.id = "root";
      this.name = "root";
    } else {
      this.id = this.path.join(Node.separator);
      this.name = maybeName;
    }
    this.length = this.path.length;
  }

  parent(): Node {
    if (this.path.length === 0)
      throw new Error("Cannot get parent of root node");
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
  selected: Writable<Node | null>;
  highlighted: Writable<Node | null>;
  dropEffect: Writable<"link" | "move" | "none">;
  dragged: Writable<Node | null>;
  expanded: Writable<Set<string>>;
  nodes: Readable<Node[]>;
  parents: Readable<Parents>;

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

    this.selected = writable<Node | null>(null);
    this.highlighted = writable<Node | null>(null);
    this.dropEffect = writable<"link" | "move" | "none">("none");
    this.dragged = writable<Node | null>(null);

    const expandedLocalStorageKey = `expanded-nodes-${projectId}`;
    this.expanded = storable<Set<string>>(
      expandedLocalStorageKey,
      new Set(),
      (json: string) => new Set<string>(JSON.parse(json)),
      (value) => JSON.stringify(Array.from(value)),
    );

    this.nodes = derived([this.expanded, this.events], ([expanded]) =>
      this.#flatten(new Node([]), [], expanded),
    );

    this.parents = derived([this.events], () => this.#toParents());
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

  #flatten(node: Node, nodes: Node[], expanded: Set<string>): Node[] {
    const task = this.yGraph.get(node.name);
    if (task && (node.length < 1 || expanded.has(node.id))) {
      for (const childName of task.get("children") as Y.Array<string>) {
        const child = node.concat(childName);
        nodes.push(child);
        this.#flatten(child, nodes, expanded);
      }
    }
    return nodes;
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
    const task = this.getTask(node.parent().name);
    return task.children.indexOf(node.name);
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

  linkNode(nodeId: string, parentId: string, offset: number) {
    this.yDoc.transact(() => {
      const yParent = this.yGraph.get(parentId);
      if (!yParent) throw new Error(`Task ${parentId} is not in the graph`);
      const yChildren = yParent.get("children") as Y.Array<string>;
      yChildren.insert(offset, [nodeId]);
    });
  }

  #unlinkNode(node: Node) {
    const nodeId = node.name;
    const parentId = node.parent().name;
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

  insertNode(
    parentId: string,
    offset: number,
    name: string,
    user: User,
  ): string {
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
      const yParent = this.yGraph.get(parentId)!;
      const yChildren = yParent.get("children") as Y.Array<string>;
      yChildren.insert(offset, [taskId]);
    });
    return taskId;
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
