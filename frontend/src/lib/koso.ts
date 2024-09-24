import { List, Map, Record, Set } from "immutable";
import * as decoding from "lib0/decoding";
import * as encoding from "lib0/encoding";
import { toast } from "svelte-sonner";
import {
  derived,
  get,
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

type NodeProps = { path: List<string> };
const NodeRecord = Record<NodeProps>({ path: List() });

export class Node extends NodeRecord {
  static get separator() {
    return "/";
  }

  static parse(id: string): Node {
    return new Node({ path: List(id.split(Node.separator)) });
  }

  get id(): string {
    return this.path.size !== 0 ? this.path.join(Node.separator) : "root";
  }

  get name(): string {
    return this.path.last("root");
  }

  get length(): number {
    return this.path.size;
  }

  ancestor(generation: number): Node {
    return new Node({ path: this.path.slice(0, -generation) });
  }

  get parent(): Node {
    return this.ancestor(1);
  }

  child(name: string): Node {
    return new Node({ path: this.path.push(name) });
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
  statusTime: number | null;
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type YEvent = Y.YEvent<any>;
export type Graph = { [id: string]: Task };
export type Nodes = Map<string, Node>;

export type Progress = {
  numer: number;
  denom: number;
};

export type YTaskProps = Y.Array<string> | string | number | null;
export type YTask = Y.Map<YTaskProps>;

export class Koso {
  yDoc: Y.Doc;
  undoManager: Y.UndoManager;
  yGraph: Y.Map<YTask>;
  yIndexedDb: IndexeddbPersistence;
  clientMessageHandler: (message: Uint8Array) => void;

  debug: Writable<boolean>;
  events: Readable<YEvent[]>;
  selected: Writable<Node | null>;
  editing: Writable<boolean>;
  highlighted: Writable<string | null>;
  dropEffect: Writable<"copy" | "move" | "none">;
  dragged: Writable<Node | null>;
  expanded: Writable<Set<Node>>;
  nodes: Readable<List<Node>>;
  parents: Readable<Map<string, string[]>>;

  constructor(projectId: string, yDoc: Y.Doc) {
    this.yDoc = yDoc;
    this.yGraph = yDoc.getMap("graph");
    this.undoManager = new Y.UndoManager(this.yGraph);
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
    this.parents = derived(this.events, () => Map(this.#toParents()));

    this.selected = writable<Node | null>(null);
    this.editing = writable<boolean>(false);
    this.highlighted = writable<string | null>(null);
    this.dropEffect = writable<"copy" | "move" | "none">("none");
    this.dragged = writable<Node | null>(null);

    const expandedLocalStorageKey = `expanded-nodes-${projectId}`;
    this.expanded = storable<Set<Node>>(
      expandedLocalStorageKey,
      Set(),
      (json: string) => Set(JSON.parse(json).map(Node.parse)),
      (nodes) => JSON.stringify(nodes.map((node) => node.id)),
    );

    this.nodes = derived(
      [this.expanded, this.events],
      ([expanded]): List<Node> => {
        // The nodes store is consistently initialized prior to the ygraph
        // being loaded, but #flatten expects the presence of at least a
        // "root" task. Handle the situation here to avoid generating warnings
        // in #flatten.
        if (this.yGraph.size === 0) {
          return List();
        }
        return this.#flatten(new Node(), expanded);
      },
    );
  }

  get root(): Node {
    return new Node();
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
          this.upsertRoot();
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

  toJSON(): Graph {
    return this.yGraph.toJSON();
  }

  expand(node: Node) {
    this.expanded.update(($expanded) => $expanded.add(node));
  }

  collapse(node: Node) {
    this.expanded.update(($expanded) => $expanded.delete(node));
  }

  #flatten(
    node: Node,
    expanded: Set<Node>,
    nodes: List<Node> = List(),
  ): List<Node> {
    if (!this.isVisible(node)) {
      return nodes;
    }

    const task = this.yGraph.get(node.name);
    if (task) {
      nodes = nodes.push(node);
      if (node.length < 1 || expanded.has(node)) {
        (task.get("children") as Y.Array<string>).forEach((name) => {
          nodes = this.#flatten(node.child(name), expanded, nodes);
        });
      }
    } else {
      console.warn(`Missing child task ${node.id}`);
    }
    return nodes;
  }

  #toParents(): { [id: string]: string[] } {
    const parents: { [id: string]: string[] } = {};
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

  getOffset(node: Node): number {
    const offset = this.getChildren(node.parent.name).indexOf(node.name);
    if (offset < 0) throw new Error(`Node ${node.name} not found in parent`);
    return offset;
  }

  getPrevPeer(node: Node): Node | null {
    const parent = node.parent;
    const peers = this.getChildren(parent.name);
    const offset = peers.indexOf(node.name);
    if (offset === -1) throw new Error(`Node ${node.name} not found in parent`);
    const prevPeerOffset = offset - 1;
    if (prevPeerOffset < 0) {
      return null;
    }

    for (const peer of peers.slice(0, offset).reverse()) {
      const peerNode = parent.child(peer);
      if (this.isVisible(peerNode)) {
        return peerNode;
      }
    }
    return null;
  }

  getNextPeer(node: Node): Node | null {
    const parent = node.parent;
    const peers = this.getChildren(parent.name);
    const offset = peers.indexOf(node.name);
    if (offset === -1) throw new Error(`Node ${node.name} not found in parent`);
    const nextPeerOffset = offset + 1;
    if (nextPeerOffset > peers.length - 1) {
      return null;
    }
    for (const peer of peers.slice(offset + 1)) {
      const peerNode = parent.child(peer);
      if (this.isVisible(peerNode)) {
        return peerNode;
      }
    }
    return null;
  }

  #getYTask(taskId: string): YTask {
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
    const allChildTaskIds = Set<string>().withMutations((allChildTaskIds) => {
      for (const task of this.yGraph.values()) {
        for (const childTaskId of task.get("children") as Y.Array<string>) {
          allChildTaskIds.add(childTaskId);
        }
      }
    });

    const allTaskIds = Set<string>().withMutations((allTaskIds) => {
      for (const taskId of this.yGraph.keys()) {
        allTaskIds.add(taskId);
      }
    });

    return allTaskIds.subtract(allChildTaskIds).toArray();
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

  upsertRoot() {
    this.upsert({
      id: "root",
      num: "0",
      name: "Root",
      children: [],
      reporter: null,
      assignee: null,
      status: null,
      statusTime: null,
    });
  }

  upsert(task: Task) {
    this.yGraph.set(
      task.id,
      new Y.Map<YTaskProps>([
        ["id", task.id],
        ["num", task.num],
        ["name", task.name],
        ["children", Y.Array.from(task.children)],
        ["reporter", task.reporter],
        ["assignee", task.assignee],
        ["status", task.status],
        ["statusTime", task.statusTime],
      ]),
    );
  }

  deleteNode(node: Node) {
    const subtreeTaskIds = this.#collectSubtreeTaskIds(node.name);

    // Find all of the tasks that will become orphans when `node`
    // is unlinked. In other words, tasks whose only parents are also in the sub-tree
    // being deleted.
    const parents = this.#toParents();
    const stack = [node.name];
    let orphanTaskIds = Set<string>();
    let visited = Set<string>();
    while (stack.length > 0) {
      const taskId = stack.pop();
      if (!taskId || visited.has(taskId)) {
        continue;
      }
      visited = visited.add(taskId);

      // Don't delete tasks that are linked to outside of the target sub-tree.
      const linkedElseWhere = parents[taskId].find((parentTaskId) => {
        const isTargetNode =
          taskId === node.name && parentTaskId === node.parent.name;
        const parentInSubtree = subtreeTaskIds.has(parentTaskId);
        return !isTargetNode && !parentInSubtree;
      });
      if (linkedElseWhere) {
        continue;
      }

      orphanTaskIds = orphanTaskIds.add(taskId);
      for (const childTaskId of this.getChildren(taskId)) {
        stack.push(childTaskId);
      }
    }

    this.yDoc.transact(() => {
      // Unlink the target node.
      const yParent = this.yGraph.get(node.parent.name);
      if (!yParent)
        throw new Error(`Task ${node.parent.name} is not in the graph`);
      const yParentsChildren = yParent.get("children") as Y.Array<string>;
      const childIndex = yParentsChildren.toArray().indexOf(node.name);
      if (childIndex < 0)
        throw new Error(
          `Task ${node.name} is not in the children of ${node.parent.name}`,
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
    return Set<string>().withMutations((subtreeTaskIds) => {
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
    });
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
    return node.parent.name === parent || this.canLink(node, parent);
  }

  moveNode(node: Node, parent: string, offset: number) {
    if (offset < 0) {
      throw new Error(`Cannot move  ${node.name} to negative offset ${offset}`);
    }
    if (!this.canMove(node, parent))
      throw new Error(`Cannot move ${node.name} to ${parent}`);
    const srcOffset = this.getOffset(node);
    this.yDoc.transact(() => {
      const srcParentName = node.parent.name;
      const ySrcChildren = this.#getYChildren(srcParentName);
      ySrcChildren.delete(srcOffset);

      if (srcParentName === parent && srcOffset < offset) {
        offset -= 1;
      }
      this.#insertChild(node.name, parent, offset);
    });
  }

  moveNodeUp(node: Node) {
    const prevPeer = this.getPrevPeer(node);
    if (!prevPeer) return;
    const offset = this.getOffset(prevPeer);
    this.moveNode(node, node.parent.name, offset);
    this.selected.set(node);
  }

  moveNodeDown(node: Node) {
    const nextPeer = this.getNextPeer(node);
    if (!nextPeer) return;
    const offset = this.getOffset(nextPeer);
    this.moveNode(node, node.parent.name, offset + 1);
    this.selected.set(node);
  }

  moveNodeRowUp(node: Node) {
    const nodes = get(this.nodes);
    const index = nodes.findIndex((n) => n.equals(node));
    if (index === -1)
      throw new Error(
        `Could not find node ${node.path} in ${nodes.map((n) => n.path)}`,
      );
    let adjIndex = index - 1;

    const debug = get(this.debug);
    let attempts = 0;
    const maybeMove = (newParent: Node, newOffset: number) => {
      if (debug) {
        console.debug(
          `Trying to move up: newParent: ${newParent.id}, offset: ${newOffset}`,
        );
      }
      if (!this.canMove(node, newParent.name)) {
        attempts++;
        return false;
      }
      this.moveNode(node, newParent.name, newOffset);
      this.selected.set(newParent.child(node.name));
      if (attempts > 0) {
        toast.info(
          `Skipped over ${attempts} position${attempts > 1 ? "s" : ""} to avoid collision with existing task`,
        );
      }
      return true;
    };
    const nearestGrandchildAncestor = (n: Node, targetGrandParent: Node) => {
      while (!n.parent.parent.equals(targetGrandParent)) {
        if (n.length == 0) {
          throw new Error("No more parents");
        }
        n = n.parent;
      }
      return n;
    };

    const initPrevAdj = adjIndex == 0 ? null : nodes.get(adjIndex);
    if (!initPrevAdj) {
      // The node in the "zeroth" position is the root, don't move it.
      return;
    }

    let insertionTarget: Node | null = null;
    if (
      !initPrevAdj.parent.equals(node.parent) &&
      !initPrevAdj.equals(node.parent)
    ) {
      insertionTarget = nearestGrandchildAncestor(initPrevAdj, node.parent);
    }

    while (true) {
      const adj = adjIndex == 0 ? null : nodes.get(adjIndex);
      if (!adj) {
        toast.info("Cannot move up without conflict.");
        return;
      }

      if (!insertionTarget) {
        if (maybeMove(adj.parent, this.getOffset(adj))) {
          return;
        }

        adjIndex--;
        const adjAdj = adjIndex == 0 ? null : nodes.get(adjIndex);
        if (
          adjAdj &&
          !adjAdj.parent.equals(adj.parent) &&
          !adjAdj.equals(adj.parent)
        ) {
          insertionTarget = nearestGrandchildAncestor(adjAdj, adj.parent);
        }
      } else {
        if (
          maybeMove(insertionTarget.parent, this.getOffset(insertionTarget) + 1)
        ) {
          return;
        }

        if (insertionTarget.equals(adj)) {
          insertionTarget = null;
        } else {
          insertionTarget = nearestGrandchildAncestor(
            adj,
            insertionTarget.parent,
          );
        }
      }
    }
  }

  moveNodeRowDown(node: Node) {
    const nodes = get(this.nodes);
    const index = nodes.findIndex((n) => n.equals(node));
    if (index === -1)
      throw new Error(
        `Could not find node ${node.path} in ${nodes.map((n) => n.path)}`,
      );
    let adjIndex = index + 1;

    const debug = get(this.debug);
    let attempts = 0;
    const maybeMove = (newParent: Node, newOffset: number) => {
      if (debug) {
        console.debug(
          `Trying to move down: newParent: ${newParent.id}, offset: ${newOffset}`,
        );
      }
      if (!this.canMove(node, newParent.name)) {
        attempts++;
        return false;
      }
      this.moveNode(node, newParent.name, newOffset);
      this.selected.set(newParent.child(node.name));
      if (attempts > 0) {
        toast.info(
          `Skipped over ${attempts} position${attempts > 1 ? "s" : ""} to avoid collision with existing task`,
        );
      }
      return true;
    };

    // Find the next node that this node is not an ancestor of,
    // either a direct peer or a peer of an ancestor.
    let initAdj = null;
    for (; adjIndex < nodes.size; adjIndex++) {
      const n = nodes.get(adjIndex);
      if (!n) throw new Error(`Node at ${adjIndex} does not exist`);
      if (!n.id.startsWith(node.id)) {
        initAdj = n;
        break;
      }
    }
    // There's no where to move to if this node
    // is the last node and an immediate child of the root,
    if (!initAdj && node.parent.equals(nodes.get(0))) {
      return;
    }

    let insertionTarget: Node | null = null;
    if (!initAdj || !initAdj.parent.equals(node.parent)) {
      insertionTarget = node.parent;
    }

    while (true) {
      const adj = nodes.get(adjIndex);
      if (!adj) {
        if (!insertionTarget) throw new Error("Expected insertionTarget.");
        if (insertionTarget.equals(nodes.get(0))) {
          toast.info("Cannot move down without conflict.");
          return;
        }

        if (
          maybeMove(insertionTarget.parent, this.getOffset(insertionTarget) + 1)
        ) {
          return;
        }
        insertionTarget = insertionTarget.parent;
      } else if (!insertionTarget) {
        const adjAdj = nodes.get(adjIndex + 1);
        const adjHasChild = adjAdj && adjAdj.parent.equals(adj);
        if (adjHasChild) {
          if (maybeMove(adj, 0)) {
            return;
          }
          adjIndex++;
        } else {
          if (maybeMove(adj.parent, this.getOffset(adj) + 1)) {
            return;
          }

          if (!adjAdj || (adjAdj && !adjAdj.parent.equals(adj.parent))) {
            insertionTarget = adj.parent;
          }
          adjIndex++;
        }
      } else {
        if (
          maybeMove(insertionTarget.parent, this.getOffset(insertionTarget) + 1)
        ) {
          return;
        }

        if (
          insertionTarget.equals(nodes.get(0)) ||
          insertionTarget.parent.equals(adj.parent)
        ) {
          insertionTarget = null;
        } else {
          insertionTarget = insertionTarget.parent;
        }
      }
    }
  }

  canIndentNode(node: Node) {
    const peer = this.getPrevPeer(node);
    return peer && this.canMove(node, peer.name);
  }

  indentNode(node: Node) {
    const peer = this.getPrevPeer(node);
    if (!peer || !this.canIndentNode(node)) return;
    this.moveNode(node, peer.name, this.getChildCount(peer.name));
    this.expand(peer);
    this.selected.set(peer.child(node.name));
  }

  canUndentNode(node: Node) {
    if (node.length < 2) return false;
    return this.canMove(node, node.parent.parent.name);
  }

  undentNode(node: Node) {
    if (!this.canUndentNode(node)) return;
    const parent = node.parent;
    const offset = this.getOffset(parent);
    this.moveNode(node, parent.parent.name, offset + 1);
    this.selected.set(parent.parent.child(node.name));
  }

  insertNode(
    parent: Node,
    offset: number,
    user: User,
    name: string = "",
  ): Node {
    const taskId = this.newId();
    this.yDoc.transact(() => {
      this.upsert({
        id: taskId,
        num: this.newNum(),
        name: name,
        children: [],
        reporter: user.email,
        assignee: null,
        status: null,
        statusTime: null,
      });
      this.#insertChild(taskId, parent.name, offset);
    });
    this.editing.set(true);
    this.selected.set(parent.child(taskId));
    return parent.child(taskId);
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

  setTaskStatus(node: Node, status: Status) {
    const taskId = node.name;
    this.yDoc.transact(() => {
      const yNode = this.yGraph.get(taskId);
      if (!yNode) throw new Error(`Task ${taskId} is not in the graph`);
      if (yNode.get("status") === status) return;

      yNode.set("status", status);
      yNode.set("statusTime", Date.now());
      // When a task is marked done, make it the last child
      // and select an adjacent peer.
      if (status === "Done") {
        const peer = this.getPrevPeer(node) || this.getNextPeer(node);
        if (peer) {
          this.selected.set(peer);
        }

        for (const parentTask of this.yGraph.values()) {
          const children = parentTask.get("children") as Y.Array<string>;
          const index = children.slice().indexOf(taskId);
          if (index !== -1) {
            children.delete(index);
            children.push([taskId]);
          }
        }
      }
    });
  }

  getProgress(taskId: string): Progress {
    const task = this.getTask(taskId);
    if (task.children.length === 0) {
      return task.status === "Done"
        ? { numer: 1, denom: 1 }
        : { numer: 0, denom: 1 };
    }
    const result = { numer: 0, denom: 0 };
    task.children.forEach((taskId) => {
      const childProgress = this.getProgress(taskId);
      result.numer += childProgress.numer;
      result.denom += childProgress.denom;
    });
    return result;
  }

  undo() {
    this.undoManager.undo();
  }

  redo() {
    this.undoManager.redo();
  }

  isVisible(node: Node) {
    // TODO: Implement this for realz.
    const task = this.getTask(node.name);
    return task.name !== "filtered";
  }
}
