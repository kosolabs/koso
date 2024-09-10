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
  index: number;

  static get separator() {
    return "/";
  }

  static parse(id: string): string[] {
    return id.split(Node.separator);
  }

  static id(path: string[]) {
    return path.join(Node.separator) || "root";
  }

  get id() {
    return Node.id(this.path);
  }

  static ancestorName(path: string[], generation: number) {
    return path.at(-1 - generation) ?? "root";
  }

  ancestorName(generation: number) {
    return Node.ancestorName(this.path, generation);
  }

  static name(path: string[]) {
    return this.ancestorName(path, 0);
  }

  get name(): string {
    return Node.name(this.path);
  }

  static parentName(path: string[]) {
    return this.ancestorName(path, 1);
  }

  get parentName(): string {
    return Node.parentName(this.path);
  }

  get length(): number {
    return this.path.length;
  }

  static concat(path: string[], child: string): string {
    return Node.id(path.concat(child));
  }

  constructor(koso: Koso, path: string[], offset: number, index: number = 0) {
    this.#koso = koso;
    this.path = path;
    this.offset = offset;
    this.index = index;
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
export type Nodes = Map<string, Node>;
export type Parents = { [id: string]: string[] };

export class Koso {
  yDoc: Y.Doc;
  yGraph: Y.Map<Y.Map<Y.Array<string> | string | null>>;
  yIndexedDb: IndexeddbPersistence;
  clientMessageHandler: (message: Uint8Array) => void;

  debug: Writable<boolean>;
  events: Readable<YEvent[]>;
  selectedId: Writable<string | null>;
  highlightedId: Writable<string | null>;
  dropEffect: Writable<"link" | "move" | "none">;
  draggedId: Writable<string | null>;
  expanded: Writable<Set<string>>;
  parents: Readable<Parents>;
  nodeIds: string[] = [];
  nodes: Readable<Nodes>;
  #nodes: Nodes = new Map();
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

    this.debug = storable<boolean>("debug", false);
    this.events = readable<YEvent[]>([], (set) => {
      const observer = (events: YEvent[]) => set(events);
      this.observe(observer);
      return () => this.unobserve(observer);
    });

    this.selectedId = writable<string | null>(null);
    this.highlightedId = writable<string | null>(null);
    this.dropEffect = writable<"link" | "move" | "none">("none");
    this.draggedId = writable<string | null>(null);

    const expandedLocalStorageKey = `expanded-nodes-${projectId}`;
    this.expanded = storable<Set<string>>(
      expandedLocalStorageKey,
      new Set(),
      (json: string) => new Set<string>(JSON.parse(json)),
      (value) => JSON.stringify(Array.from(value)),
    );

    this.nodes = derived([this.expanded, this.events], ([expanded]) =>
      this.#flatten(new Node(this, [], 0), expanded),
    );
    this.parents = derived([this.events], () => this.#toParents());

    this.unsubscribe = this.nodes.subscribe(($nodes) => {
      this.#nodes = $nodes;
      this.nodeIds = Array.from($nodes.keys());
    });
  }

  get nodelen(): number {
    return this.#nodes.size;
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

  expand(id: string) {
    this.expanded.update(($expanded) => $expanded.add(id));
  }

  collapse(id: string) {
    this.expanded.update(($expanded) => {
      $expanded.delete(id);
      return $expanded;
    });
  }

  #flatten(node: Node, expanded: Set<string>, nodes: Nodes = new Map()): Nodes {
    const task = this.yGraph.get(node.name);
    if (task) {
      nodes.set(node.id, node);
      if (node.length < 1 || expanded.has(node.id)) {
        (task.get("children") as Y.Array<string>).forEach((name, offset) => {
          const child = new Node(
            this,
            node.path.concat(name),
            offset,
            nodes.size,
          );
          this.#flatten(child, expanded, nodes);
        });
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

  getNode(id: string): Node {
    const result = this.#nodes.get(id);
    if (!result) throw new Error(`Node ID ${id} not found in nodes`);
    return result;
  }

  getNodeId(index: number): string {
    if (index < 0 || index >= this.nodeIds.length)
      throw new Error(`Node index ${index} out of bounds`);
    return this.nodeIds[index];
  }

  getParent(node: Node): Node {
    if (node.path.length === 0)
      throw new Error("Cannot get parent of root node");
    return this.getNode(Node.id(node.path.slice(0, -1)));
  }

  getPrevPeer(node: Node): Node {
    const peers = this.getChildren(node.parentName);
    return this.getNode(
      Node.id(node.path.slice(0, -1).concat(peers[node.offset - 1])),
    );
  }

  getChild(node: Node, childName: string): Node {
    return this.getNode(Node.id(node.path.concat(childName)));
  }

  #getYTask(taskId: string): Y.Map<string | Y.Array<string> | null> {
    const yTask = this.yGraph.get(taskId);
    if (!yTask) throw new Error(`Task ID ${taskId} not found in yGraph`);
    return yTask;
  }

  getTask(taskId: string): Task {
    return this.#getYTask(taskId).toJSON() as Task;
  }

  #getYChildren(taskId: string): Y.Array<string> {
    return this.#getYTask(taskId).get("children") as Y.Array<string>;
  }

  getChildren(taskId: string): string[] {
    return this.#getYChildren(taskId).toArray();
  }

  getChildCount(taskId: string): number {
    return this.#getYChildren(taskId).length;
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

  deleteNode(node: Node) {
    const subtreeTaskIds = this.#collectSubtreeTaskIds(node.name);

    // Find all of the tasks that will become orphans when `node`
    // is unlinked. In other words, tasks whose only parents are also in the sub-tree
    // being deleted.
    const parents = this.#toParents();
    const orphanTaskIds = new Set<string>();
    const visited = new Set<string>();
    const stack = [node.name];
    while (stack.length > 0) {
      const taskId = stack.pop();
      if (!taskId || visited.has(taskId)) {
        continue;
      }
      visited.add(taskId);

      // Don't delete tasks that are linked to outside of the target sub-tree.
      const linkedElseWhere = parents[taskId].find((parentTaskId) => {
        const isTargetNode =
          taskId === node.name && parentTaskId === node.parentName;
        const parentInSubtree = subtreeTaskIds.has(parentTaskId);
        return !isTargetNode && !parentInSubtree;
      });
      if (linkedElseWhere) {
        continue;
      }

      orphanTaskIds.add(taskId);
      for (const childTaskId of this.getChildren(taskId)) {
        stack.push(childTaskId);
      }
    }

    this.yDoc.transact(() => {
      // Unlink the target node.
      const yParent = this.yGraph.get(node.parentName);
      if (!yParent)
        throw new Error(`Task ${node.parentName} is not in the graph`);
      const yParentsChildren = yParent.get("children") as Y.Array<string>;
      const childIndex = yParentsChildren.toArray().indexOf(node.name);
      if (childIndex < 0)
        throw new Error(
          `Task ${node.name} is not in the children of ${node.parentName}`,
        );
      yParentsChildren.delete(childIndex);

      // Delete all of the now orphaned tasks.
      for (const taskId of orphanTaskIds) {
        this.yGraph.delete(taskId);
      }
    });
  }

  // Collect all task IDs in the sub-tree starting at `taskId`.
  #collectSubtreeTaskIds(taskId: string) {
    const subtreeTaskIds = new Set<string>();
    const stack = [taskId];
    while (stack.length > 0) {
      const taskId = stack.pop();
      if (!taskId) {
        continue;
      }
      subtreeTaskIds.add(taskId);
      for (const childTaskId of this.getChildren(taskId)) {
        if (!subtreeTaskIds.has(childTaskId)) {
          stack.push(childTaskId);
        }
      }
    }
    return subtreeTaskIds;
  }

  #hasCycle(parent: string, child: string): boolean {
    if (child === parent) {
      return true;
    }
    for (const next of this.getChildren(child)) {
      if (this.#hasCycle(parent, next)) {
        return true;
      }
    }
    return false;
  }

  #hasChild(parent: string, child: string): boolean {
    return this.getChildren(parent).indexOf(child) !== -1;
  }

  #insertChild(child: string, parent: string, offset: number) {
    if (this.#hasCycle(parent, child)) {
      throw new Error(`Inserting ${child} under ${parent} introduces a cycle`);
    }

    if (this.#hasChild(parent, child)) {
      throw new Error(`Parent task ${parent} already contains ${child}`);
    }

    const yChildren = this.#getYChildren(parent);
    yChildren.insert(offset, [child]);
  }

  canLink(node: Node, parent: string): boolean {
    return (
      !this.#hasCycle(parent, node.name) && !this.#hasChild(parent, node.name)
    );
  }

  linkNode(node: Node, parent: string, offset: number) {
    if (!this.canLink(node, parent))
      throw new Error(`Cannot link ${node.name} to ${parent}`);
    this.yDoc.transact(() => {
      this.#insertChild(node.name, parent, offset);
    });
  }

  canMove(node: Node, parent: string): boolean {
    return node.parentName === parent || this.canLink(node, parent);
  }

  moveNode(node: Node, parent: string, offset: number) {
    if (!this.canMove(node, parent))
      throw new Error(`Cannot move ${node.name} to ${parent}`);
    this.yDoc.transact(() => {
      const srcParentName = node.parentName;
      const ySrcChildren = this.#getYChildren(srcParentName);
      ySrcChildren.delete(node.offset);

      if (srcParentName === parent && node.offset < offset) {
        offset -= 1;
      }
      this.#insertChild(node.name, parent, offset);
    });
  }

  moveNodeUp(node: Node) {
    if (node.offset < 1) return;
    this.moveNode(node, node.parentName, node.offset - 1);
  }

  moveNodeDown(node: Node) {
    if (node.offset >= this.getChildCount(node.parentName) - 1) return;
    this.moveNode(node, node.parentName, node.offset + 2);
  }

  canIndentNode(node: Node) {
    const peer = this.getPrevPeer(node);
    return this.canMove(node, peer.name);
  }

  indentNode(node: Node) {
    if (node.offset < 1) return;
    const peer = this.getPrevPeer(node);
    if (!this.canIndentNode(node)) return;
    this.moveNode(node, peer.name, this.getChildCount(peer.name));
    this.expand(peer.id);
    this.selectedId.set(Node.concat(peer.path, node.name));
  }

  canUndentNode(node: Node) {
    const parent = node.parent();
    return this.canMove(node, parent.parentName);
  }

  undentNode(node: Node) {
    if (node.length < 2) return;
    const parent = node.parent();
    if (!this.canUndentNode(node)) return;
    this.moveNode(node, parent.parentName, parent.offset + 1);
    this.selectedId.set(Node.concat(parent.parent().path, node.name));
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
      this.#insertChild(taskId, parent.name, offset);
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
