import { List, Map, Record, Set } from "immutable";
import * as decoding from "lib0/decoding";
import * as encoding from "lib0/encoding";
import { toast } from "svelte-sonner";
import { v4 as uuidv4 } from "uuid";
import { IndexeddbPersistence } from "y-indexeddb";
import * as Y from "yjs";
import type { User } from "./auth.svelte";
import { useLocalStorage, type Storable } from "./stores.svelte";

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
  // Time, in milliseconds since the unix epoch,
  // when the `status` field was last modified.
  statusTime: number | null;
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type YEvent = Y.YEvent<any>;
export type Graph = { [id: string]: Task };
export type Nodes = Map<string, Node>;

export type Progress = {
  inProgress: number;
  done: number;
  total: number;
  lastStatusTime: number | null;
};

export type YTaskProps = Y.Array<string> | string | number | null;
export type YTask = Y.Map<YTaskProps>;

export type SyncState = {
  // True when the indexed DB is sync'd with the Koso doc.
  indexedDbSync: boolean;
  // True when state from the server is sync'd with the Koso doc.
  serverSync: boolean;
};

export class Koso {
  projectId: string;
  yDoc: Y.Doc;
  yGraph: Y.Map<YTask>;
  yUndoManager: Y.UndoManager;
  yIndexedDb: IndexeddbPersistence;
  clientMessageHandler: (message: Uint8Array) => void;

  #selected: Node | null = $state(null);
  focus: boolean = $state(false);
  highlighted: string | null = $state(null);
  dragged: Node | null = $state(null);
  dropEffect: "copy" | "move" | "none" = $state("none");

  #debug: Storable<boolean>;
  #observer: (events: YEvent[]) => void;
  #events: YEvent[] = $state.raw([]);
  #graph: Graph = $derived.by(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    this.#events;
    return this.toJSON();
  });
  #expanded: Storable<Set<Node>>;
  #showDone: Storable<boolean>;
  #parents: Map<string, string[]> = $derived.by(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    this.#events;
    return this.#toParents();
  });
  #nodes: List<Node> = $derived.by(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    this.#events;
    // The nodes store is consistently initialized prior to the ygraph
    // being loaded, but #flatten expects the presence of at least a
    // "root" task. Handle the situation here to avoid generating warnings
    // in #flatten.
    if (this.yGraph.size === 0) {
      return List();
    }
    return this.#flatten(new Node(), this.expanded, this.showDone);
  });
  #syncState: SyncState = $state({
    indexedDbSync: false,
    serverSync: false,
  });

  constructor(projectId: string, yDoc: Y.Doc) {
    this.projectId = projectId;
    this.yDoc = yDoc;
    this.yGraph = yDoc.getMap("graph");
    this.yUndoManager = new Y.UndoManager(this.yGraph);
    // Save and restore node selection on undo/redo.
    this.yUndoManager.on("stack-item-added", (event) => {
      event.stackItem.meta.set("selected-node", this.selected);
    });
    this.yUndoManager.on("stack-item-popped", (event) => {
      const selected = event.stackItem.meta.get("selected-node");
      if (selected === null || selected.constructor === Node) {
        this.selected = selected;
      } else {
        console.warn(
          `Unexpectedly found non-node "selected-node" stack item: ${selected}`,
        );
        this.selected = null;
      }
    });
    this.yIndexedDb = new IndexeddbPersistence(`koso-${projectId}`, this.yDoc);
    this.clientMessageHandler = () => {
      throw new Error("Client message handler was invoked but was not set");
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

    this.#debug = useLocalStorage("debug", false);

    this.#observer = (events: YEvent[]) => (this.#events = events);
    this.observe(this.#observer);

    this.#expanded = useLocalStorage<Set<Node>>(
      `expanded-nodes-${projectId}`,
      Set(),
      {
        decode: (json: string) => Set(JSON.parse(json).map(Node.parse)),
        encode: (nodes) => JSON.stringify(nodes.map((node) => node.id)),
      },
    );

    this.#showDone = useLocalStorage<boolean>(`show-done-${projectId}`, false);

    this.yIndexedDb.whenSynced.then(() => {
      this.#syncState.indexedDbSync = true;
    });
  }

  destroy() {
    this.unobserve(this.#observer);
  }

  get selected(): Node | null {
    return this.#selected;
  }

  set selected(value: Node | null) {
    this.#selected = value;
    this.focus = true;
  }

  get root(): Node {
    return new Node();
  }

  get debug(): boolean {
    return this.#debug.value;
  }

  set debug(value: boolean) {
    this.#debug.value = value;
  }

  get graph(): Graph {
    return this.#graph;
  }

  get expanded(): Set<Node> {
    return this.#expanded.value;
  }

  set expanded(value: Set<Node>) {
    this.#expanded.value = value;
  }

  get showDone(): boolean {
    return this.#showDone.value;
  }

  set showDone(value: boolean) {
    this.#showDone.value = value;
  }

  get parents(): Map<string, string[]> {
    return this.#parents;
  }

  get nodes(): List<Node> {
    return this.#nodes;
  }

  get syncState(): SyncState {
    return this.#syncState;
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
        this.#syncState.serverSync = true;
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

  canExpand(node: Node) {
    return !this.expanded.contains(node) && this.getChildCount(node.name) > 0;
  }

  expand(node: Node) {
    this.expanded = this.expanded.add(node);
  }

  canCollapse(node: Node) {
    return this.expanded.contains(node) && this.getChildCount(node.name) > 0;
  }

  collapse(node: Node) {
    this.expanded = this.expanded.delete(node);
  }

  #flatten(
    node: Node,
    expanded: Set<Node>,
    showDone: boolean,
    nodes: List<Node> = List(),
  ): List<Node> {
    const task = this.yGraph.get(node.name);
    if (task) {
      nodes = nodes.push(node);
      if (node.length < 1 || expanded.has(node)) {
        (task.get("children") as Y.Array<string>).forEach((name) => {
          const childNode = node.child(name);
          // Apply visibility filtering here instead of at the start of #flatten
          // to ensure that the root node is always present.
          if (this.isVisible(childNode, showDone)) {
            nodes = this.#flatten(childNode, expanded, showDone, nodes);
          }
        });
      }
    } else {
      console.warn(`Missing child task ${node.id}`);
    }
    return nodes;
  }

  #toParents(): Map<string, string[]> {
    return Map<string, string[]>().withMutations((parents) => {
      for (const [parentId, task] of this.yGraph.entries()) {
        for (const childId of task.get("children") as Y.Array<string>) {
          const children = parents.get<string[]>(childId, []);
          children.push(parentId);
          parents.set(childId, children);
        }
      }
    });
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

    // Find the nearest prior peer that isn't filtered out.
    for (const peer of peers.slice(0, prevPeerOffset + 1).reverse()) {
      const peerNode = parent.child(peer);
      // TODO: This call to includes, and the one in getNextPeer, could be optimized
      // to avoid iterating over the entire nodes list repeatedly.
      if (this.nodes.includes(peerNode)) {
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

    // Find the nearest next peer that isn't filtered out.
    for (const peer of peers.slice(nextPeerOffset)) {
      const peerNode = parent.child(peer);
      if (this.nodes.includes(peerNode)) {
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
    this.yDoc.transact(() => {
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
    });
    // Prevent undoing creation of the root task.
    this.yUndoManager.clear();
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
      const parents = this.parents.get(taskId);
      if (!parents) throw new Error(`Parents missing ${taskId}`);
      const linkedElseWhere = parents.find((parentTaskId) => {
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
      const yParent = this.#getYTask(node.parent.name);
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

  canLink(task: string, parent: string): boolean {
    return !this.#hasCycle(parent, task) && !this.#hasChild(parent, task);
  }

  linkTask(task: string, parent: string, offset: number) {
    if (!this.canLink(task, parent))
      throw new Error(`Cannot link ${task} to ${parent}`);
    this.yDoc.transact(() => {
      this.#insertChild(task, parent, offset);
    });
  }

  linkNode(node: Node, parent: Node, offset: number) {
    this.linkTask(node.name, parent.name, offset);
  }

  canMove(node: Node, parent: Node): boolean {
    return (
      node.parent.name === parent.name || this.canLink(node.name, parent.name)
    );
  }

  moveNode(node: Node, parent: Node, offset: number) {
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

      if (srcParentName === parent.name && srcOffset < offset) {
        offset -= 1;
      }
      this.#insertChild(node.name, parent.name, offset);
    });
    this.selected = parent.child(node.name);
  }

  reorderNode(node: Node, offset: number) {
    this.moveNode(node, node.parent, offset);
  }

  moveNodeUp(node: Node) {
    const index = this.nodes.findIndex((n) => n.equals(node));
    if (index === -1)
      throw new Error(
        `Could not find node ${node.path} in ${this.nodes.map((n) => n.path)}`,
      );
    let adjIndex = index - 1;

    let attempts = 0;
    const maybeMove = (newParent: Node, newOffset: number) => {
      if (this.debug) {
        console.debug(
          `Trying to move up: newParent: ${newParent.id}, offset: ${newOffset}`,
        );
      }
      if (!this.canMove(node, newParent)) {
        attempts++;
        return false;
      }
      this.moveNode(node, newParent, newOffset);
      this.selected = newParent.child(node.name);
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

    const initPrevAdj = adjIndex == 0 ? null : this.nodes.get(adjIndex);
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
      const adj = adjIndex == 0 ? null : this.nodes.get(adjIndex);
      if (!adj) {
        toast.info("Cannot move up without conflict.");
        return;
      }

      if (!insertionTarget) {
        if (maybeMove(adj.parent, this.getOffset(adj))) {
          return;
        }

        adjIndex--;
        const adjAdj = adjIndex == 0 ? null : this.nodes.get(adjIndex);
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

  moveNodeDown(node: Node) {
    const index = this.nodes.findIndex((n) => n.equals(node));
    if (index === -1)
      throw new Error(
        `Could not find node ${node.path} in ${this.nodes.map((n) => n.path)}`,
      );
    let adjIndex = index + 1;

    let attempts = 0;
    const maybeMove = (newParent: Node, newOffset: number) => {
      if (this.debug) {
        console.debug(
          `Trying to move down: newParent: ${newParent.id}, offset: ${newOffset}`,
        );
      }
      if (!this.canMove(node, newParent)) {
        attempts++;
        return false;
      }
      this.moveNode(node, newParent, newOffset);
      this.selected = newParent.child(node.name);
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
    for (; adjIndex < this.nodes.size; adjIndex++) {
      const n = this.nodes.get(adjIndex);
      if (!n) throw new Error(`Node at ${adjIndex} does not exist`);
      if (!n.id.startsWith(node.id)) {
        initAdj = n;
        break;
      }
    }
    // There's no where to move to if this node
    // is the last node and an immediate child of the root,
    if (!initAdj && node.parent.equals(this.nodes.get(0))) {
      return;
    }

    let insertionTarget: Node | null = null;
    if (!initAdj || !initAdj.parent.equals(node.parent)) {
      insertionTarget = node.parent;
    }

    while (true) {
      const adj = this.nodes.get(adjIndex);
      if (!adj) {
        if (!insertionTarget) throw new Error("Expected insertionTarget.");
        if (insertionTarget.equals(this.nodes.get(0))) {
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
        const adjAdj = this.nodes.get(adjIndex + 1);
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
          insertionTarget.equals(this.nodes.get(0)) ||
          insertionTarget.parent.equals(adj.parent)
        ) {
          insertionTarget = null;
        } else {
          insertionTarget = insertionTarget.parent;
        }
      }
    }
  }

  moveNodeStart(node: Node) {
    const offset = this.getOffset(node);
    if (offset === 0) {
      const task = this.getTask(node.name);
      toast.warning(`Task ${task.num} is already at the top`);
      return;
    }
    this.reorderNode(node, 0);
  }

  moveNodeEnd(node: Node) {
    const offset = this.getOffset(node);
    const length = this.getChildCount(node.parent.name);
    if (offset === length - 1) {
      const task = this.getTask(node.name);
      toast.warning(`Task ${task.num} is already at the bottom`);
      return;
    }
    this.reorderNode(node, length);
  }

  canIndentNode(node: Node): boolean {
    const peer = this.getPrevPeer(node);
    return !!peer && this.canMove(node, peer);
  }

  indentNode(node: Node) {
    const peer = this.getPrevPeer(node);
    if (!peer || !this.canIndentNode(node)) return;
    this.moveNode(node, peer, this.getChildCount(peer.name));
    this.expand(peer);
    this.selected = peer.child(node.name);
  }

  canUndentNode(node: Node): boolean {
    if (node.length < 2) return false;
    return this.canMove(node, node.parent.parent);
  }

  undentNode(node: Node) {
    if (!this.canUndentNode(node)) return;
    const parent = node.parent;
    const offset = this.getOffset(parent);
    this.moveNode(node, parent.parent, offset + 1);
    this.selected = parent.parent.child(node.name);
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
    const node = parent.child(taskId);
    this.selected = node;
    return node;
  }

  setTaskName(taskId: string, newName: string) {
    this.yDoc.transact(() => {
      const yNode = this.#getYTask(taskId);
      if (yNode.get("name") !== newName) {
        yNode.set("name", newName);
      }
    });
  }

  setAssignee(taskId: string, assignee: User | null) {
    this.yDoc.transact(() => {
      const yNode = this.#getYTask(taskId);
      if (assignee === null && yNode.get("assignee") !== null) {
        yNode.set("assignee", null);
      } else if (assignee && assignee.email !== yNode.get("assignee")) {
        yNode.set("assignee", assignee.email);
      }
    });
  }

  setReporter(taskId: string, reporter: User | null) {
    this.yDoc.transact(() => {
      const yNode = this.#getYTask(taskId);
      if (reporter === null && yNode.get("reporter") !== null) {
        yNode.set("reporter", null);
      } else if (reporter && reporter.email !== yNode.get("reporter")) {
        yNode.set("reporter", reporter.email);
      }
    });
  }

  setTaskStatus(node: Node, status: Status, user: User) {
    const taskId = node.name;
    this.yDoc.transact(() => {
      const yNode = this.#getYTask(taskId);
      if (yNode.get("status") === status) return;

      yNode.set("status", status);
      yNode.set("statusTime", Date.now());
      // When a task is marked done, make it the last child
      // and select an adjacent peer.
      if (status === "Done") {
        const peer = this.getPrevPeer(node) || this.getNextPeer(node);
        if (peer) {
          this.selected = peer;
        }

        // If scanning all tasks ever gets slow, we could always
        // maintain a by-parent index. The same applies below.
        for (const parentTask of this.yGraph.values()) {
          const children = parentTask.get("children") as Y.Array<string>;
          const index = children.slice().indexOf(taskId);
          if (index !== -1) {
            children.delete(index);
            children.push([taskId]);
          }
        }
      }
      // When a task is marked in progress, make it the first child
      // and, if unassigned, assign to the current user
      else if (status === "In Progress") {
        for (const parentTask of this.yGraph.values()) {
          const children = parentTask.get("children") as Y.Array<string>;
          const index = children.slice().indexOf(taskId);
          if (index !== -1) {
            children.delete(index);
            children.insert(0, [taskId]);
          }
        }
        if (!yNode.get("assignee")) {
          yNode.set("assignee", user.email);
        }
      }
    });
  }

  getProgress(taskId: string): Progress {
    const task = this.getTask(taskId);
    if (task.children.length === 0) {
      switch (task.status) {
        case "Done":
          return {
            inProgress: 0,
            done: 1,
            total: 1,
            lastStatusTime: task.statusTime,
          };
        case "In Progress":
          return {
            inProgress: 1,
            done: 0,
            total: 1,
            lastStatusTime: task.statusTime,
          };
        case "Not Started":
        case null:
        case undefined:
          return {
            inProgress: 0,
            done: 0,
            total: 1,
            lastStatusTime: task.statusTime,
          };
        default:
          console.log(`Invalid status ${task.status} for task ${task.name}`);
          return {
            inProgress: 0,
            done: 0,
            total: 1,
            lastStatusTime: task.statusTime,
          };
      }
    }
    const result: Progress = {
      lastStatusTime: null,
      inProgress: 0,
      done: 0,
      total: 0,
    };
    task.children.forEach((taskId) => {
      // If performance is ever an issue for large, nested graphs,
      // we can memoize the recursive call and trade memory for time.
      const childProgress = this.getProgress(taskId);
      result.inProgress += childProgress.inProgress;
      result.done += childProgress.done;
      result.total += childProgress.total;
      if (
        childProgress.lastStatusTime &&
        (!result.lastStatusTime ||
          childProgress.lastStatusTime > result.lastStatusTime)
      ) {
        result.lastStatusTime = childProgress.lastStatusTime;
      }
    });
    return result;
  }

  undo() {
    this.yUndoManager.undo();
  }

  redo() {
    this.yUndoManager.redo();
  }

  isVisible(node: Node, showDone: boolean) {
    if (!showDone) {
      const progress = this.getProgress(node.name);
      if (progress.total === progress.done) {
        // Tasks marked done prior to the addition of statusTime
        // won't have a statusTime set. Assume they were all marked done
        // a long time ago.
        const doneTime = progress.lastStatusTime ? progress.lastStatusTime : 0;
        const threeDays = 3 * 24 * 60 * 60 * 1000;
        return Date.now() - doneTime < threeDays;
      }
    }
    return true;
  }
}
