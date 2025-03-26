import type { User } from "$lib/auth.svelte";
import { toast } from "$lib/components/ui/sonner";
import {
  parseAwarenessStateResponse,
  type Awareness,
} from "$lib/dag-table/awareness.svelte";
import { useLocalStorage, type Storable } from "$lib/stores.svelte";
import { findEntryIndex } from "$lib/utils";
import { List, Map, Record, Set } from "immutable";
import * as decoding from "lib0/decoding";
import * as encoding from "lib0/encoding";
import { v4 as uuidv4 } from "uuid";
import { IndexeddbPersistence } from "y-indexeddb";
import * as Y from "yjs";
import {
  unmanagedKinds,
  YChildrenProxy,
  YGraphProxy,
  YTaskProxy,
  type Kind,
  type Status,
  type Task,
  type YEvent,
  type YTask,
} from "../yproxy";

const MSG_SYNC = 0;

const MSG_SYNC_REQUEST = 0;
const MSG_SYNC_RESPONSE = 1;
const MSG_SYNC_UPDATE = 2;
type YMessageSync =
  | typeof MSG_SYNC_REQUEST
  | typeof MSG_SYNC_RESPONSE
  | typeof MSG_SYNC_UPDATE;

const MSG_KOSO_AWARENESS = 8;
const MSG_KOSO_AWARENESS_UPDATE = 0;
const MSG_KOSO_AWARENESS_STATE = 1;
type YMessageKosoAwareness =
  | typeof MSG_KOSO_AWARENESS_UPDATE
  | typeof MSG_KOSO_AWARENESS_STATE;

type YMessage = typeof MSG_SYNC | typeof MSG_KOSO_AWARENESS;

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

  isDescendantOf(ancestor: Node): boolean {
    return this.id.startsWith(ancestor.id);
  }

  get parent(): Node {
    return this.ancestor(1);
  }

  child(name: string): Node {
    return new Node({ path: this.path.push(name) });
  }
}

export type Nodes = Map<string, Node>;

type SelectedProps = { node: Node | null; index: number | null };
const SelectedRecord = Record<SelectedProps>({ node: null, index: null });

export class Selected extends SelectedRecord {
  constructor(props: Partial<SelectedProps>) {
    if (props.index && props.index < 0) {
      props.index = null;
    }
    super(props);
  }

  static default(): Selected {
    return DEFAULT_SELECTED;
  }

  static create(node: Node, index: number) {
    return new Selected({ node, index });
  }
}
const DEFAULT_SELECTED = new Selected({ node: null, index: null });

export class Progress {
  inProgress: number;
  done: number;
  total: number;
  lastStatusTime: number;
  status: Status;
  childrenStatus: Status | null;
  kind: Kind | null;

  constructor(props: Partial<Progress> = {}) {
    this.inProgress = props.inProgress ?? 0;
    this.done = props.done ?? 0;
    this.total = props.total ?? 0;
    this.lastStatusTime = props.lastStatusTime ?? 0;
    this.status = props.status ?? "Not Started";
    this.kind = props.kind ?? null;
    this.childrenStatus = props.childrenStatus ?? null;
  }

  isComplete(): boolean {
    return this.status === "Done";
  }

  isBlocked(): boolean {
    return this.status === "Blocked";
  }

  isChildrenIncomplete(): boolean {
    return this.childrenStatus !== null && this.childrenStatus !== "Done";
  }
}

export type SyncState = {
  // True when the indexed DB is sync'd with the Koso doc.
  indexedDbSync: boolean;
  // True when state from the server is sync'd with the Koso doc.
  serverSync: boolean;
};

export type FlattenFn = (
  node: Node,
  expanded: Set<Node>,
  showDone: boolean,
) => List<Node>;
export type VisibilityFilterFn = (node: Node, showDone: boolean) => boolean;

export class Koso {
  #projectId: string;
  #yDoc: Y.Doc;
  #yGraph: YGraphProxy;
  #yUndoManager: Y.UndoManager;
  #yIndexedDb: IndexeddbPersistence;
  #send: (message: Uint8Array) => void = () => {
    // Until we connect to the server and invoke handleClientMessage,
    // there's nothing else to do with client messages, so we simply discard them.
    // Any dropped changes will be sync'd to the server later.
    console.debug("Client message handler was invoked but was not set");
  };

  #selectedRaw: Selected = $state(Selected.default());
  #focus: boolean = $state(false);
  #highlighted: string | null = $state(null);
  #dragged: Node | null = $state(null);
  #dropEffect: "copy" | "move" | "none" = $state("none");
  #awareness: Awareness[] = $state([]);

  #debug: Storable<boolean>;
  #events: YEvent[] = $state.raw([]);
  #expanded: Storable<Set<Node>>;
  #showDone: Storable<boolean>;
  #visibilityFilterFn: VisibilityFilterFn;
  #flattenFn: FlattenFn;
  #tasks: YTaskProxy[] = $derived.by(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    this.#events;
    return Array.from(this.graph.values());
  });
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
    if (this.graph.size === 0) {
      return List();
    }
    return this.#flattenFn(new Node(), this.expanded, this.showDone);
  });
  #selected: Node | null = $derived.by(() => {
    const node = this.#selectedRaw.node;
    if (!node || this.nodes.indexOf(node) < 0) {
      return null;
    }
    return node;
  });

  #resolveIndexedDbSync: () => void = () => {};
  #indexedDbSynced = new Promise<void>(
    (resolve) => (this.#resolveIndexedDbSync = resolve),
  );
  #resolveServerSync: () => void = () => {};
  #serverSynced = new Promise<void>(
    (resolve) => (this.#resolveServerSync = resolve),
  );

  #sequence: number = 0;

  // lifecycle functions
  // i.e., init functions and helpers, event handlers, and destructors

  constructor(
    projectId: string,
    yDoc: Y.Doc,
    visibilityFilterFn?: VisibilityFilterFn,
    flattenFn?: FlattenFn,
  ) {
    this.#projectId = projectId;
    this.#yDoc = yDoc;
    const graph = yDoc.getMap<YTask>("graph");
    this.#yGraph = new YGraphProxy(graph);
    this.#yUndoManager = new Y.UndoManager(graph, {
      captureTransaction: (txn) => txn.local,
    });
    // Save and restore node selection on undo/redo.
    this.#yUndoManager.on("stack-item-added", (event) => {
      event.stackItem.meta.set("selected-node", this.selectedRaw.node);
    });
    this.#yUndoManager.on("stack-item-popped", (event) => {
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
    this.#yIndexedDb = new IndexeddbPersistence(`koso-${projectId}`, this.doc);
    this.doc.on(
      "updateV2",
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (message: Uint8Array, arg1: any, arg2: Y.Doc, txn: Y.Transaction) => {
        if (txn.local) {
          const encoder = encoding.createEncoder();
          encoding.writeVarUint(encoder, MSG_SYNC);
          encoding.writeVarUint(encoder, MSG_SYNC_UPDATE);
          encoding.writeVarUint8Array(encoder, message);
          this.#send(encoding.toUint8Array(encoder));
        }
      },
    );

    this.#debug = useLocalStorage("debug", false);

    this.#expanded = useLocalStorage<Set<Node>>(
      `expanded-nodes-${projectId}`,
      Set(),
      {
        decode: (json: string) => Set(JSON.parse(json).map(Node.parse)),
        encode: (nodes) => JSON.stringify(nodes.map((node) => node.id)),
      },
    );

    this.#showDone = useLocalStorage<boolean>(`show-done-${projectId}`, false);
    this.#visibilityFilterFn = visibilityFilterFn ?? this.#defaultIsVisible;
    this.#flattenFn = flattenFn ?? this.#defaultFlatten;

    this.#yIndexedDb.whenSynced.then(() => {
      this.#resolveIndexedDbSync();
    });

    $effect(() => {
      const observer = (events: YEvent[]) => (this.#events = events);

      this.graph.observe(observer);

      return () => {
        this.graph.unobserve(observer);
      };
    });
  }

  receive(message: Uint8Array) {
    const decoder = decoding.createDecoder(message);
    const messageType = decoding.readVarUint(decoder) as YMessage;
    if (messageType === MSG_SYNC) {
      const syncType = decoding.readVarUint(decoder) as YMessageSync;

      if (syncType === MSG_SYNC_REQUEST) {
        const encoder = encoding.createEncoder();
        const encodedStateVector = decoding.readVarUint8Array(decoder);
        encoding.writeVarUint(encoder, MSG_SYNC);
        encoding.writeVarUint(encoder, MSG_SYNC_RESPONSE);
        encoding.writeVarUint8Array(
          encoder,
          Y.encodeStateAsUpdateV2(this.doc, encodedStateVector),
        );
        this.#send(encoding.toUint8Array(encoder));
      } else if (syncType === MSG_SYNC_RESPONSE) {
        const message = decoding.readVarUint8Array(decoder);
        Y.applyUpdateV2(this.doc, message, "koso.SYNC_RESPONSE");
        if (this.graph.size === 0) {
          this.upsertRoot();
        }
        this.#resolveServerSync();
      } else if (syncType === MSG_SYNC_UPDATE) {
        const message = decoding.readVarUint8Array(decoder);
        Y.applyUpdateV2(this.doc, message, "koso.SYNC_UPDATE");
      } else {
        throw new Error(`Unknown sync type: ${syncType}`);
      }
    } else if (messageType === MSG_KOSO_AWARENESS) {
      const kosoAwarenessType = decoding.readVarUint(
        decoder,
      ) as YMessageKosoAwareness;

      if (kosoAwarenessType === MSG_KOSO_AWARENESS_UPDATE) {
        throw new Error("Unimplemented");
      } else if (kosoAwarenessType === MSG_KOSO_AWARENESS_STATE) {
        this.#awareness = parseAwarenessStateResponse(
          decoding.readVarString(decoder),
        ).filter((a) => this.clientId !== a.clientId);
      } else {
        throw new Error(`Unknown Koso awareness type: ${kosoAwarenessType}`);
      }
    } else {
      throw new Error(
        `Expected message type to be Sync (0) but was: ${messageType}`,
      );
    }
  }

  setSendAndSync(f: (message: Uint8Array) => void) {
    this.#send = f;
    this.#send(this.#encodeSyncRequest());
    this.#send(this.#encodeAwareness());
  }

  #encodeSyncRequest(): Uint8Array {
    const encoder = encoding.createEncoder();
    encoding.writeVarUint(encoder, MSG_SYNC);
    encoding.writeVarUint(encoder, MSG_SYNC_REQUEST);
    const sv = Y.encodeStateVector(this.doc);
    encoding.writeVarUint8Array(encoder, sv);
    return encoding.toUint8Array(encoder);
  }

  #encodeAwareness(): Uint8Array {
    const encoder = encoding.createEncoder();
    encoding.writeVarUint(encoder, MSG_KOSO_AWARENESS);
    encoding.writeVarUint(encoder, MSG_KOSO_AWARENESS_UPDATE);
    encoding.writeVarString(
      encoder,
      JSON.stringify({
        clientId: this.clientId,
        sequence: this.#sequence++,
        selected: this.selected ? [this.selected.id] : [],
      }),
    );
    return encoding.toUint8Array(encoder);
  }

  /** Do not call this directly. Use #flattenFn instead. */
  #defaultFlatten(
    node: Node,
    expanded: Set<Node>,
    showDone: boolean,
    nodes: List<Node> = List(),
  ): List<Node> {
    const task = this.getTask(node.name);
    nodes = nodes.push(node);
    if (node.length < 1 || expanded.has(node)) {
      task.children.forEach((name) => {
        const childNode = node.child(name);
        // Apply visibility filtering here instead of at the start of #flatten
        // to ensure that the root node is always present.
        if (this.isVisible(childNode, showDone)) {
          nodes = this.#defaultFlatten(childNode, expanded, showDone, nodes);
        }
      });
    }
    return nodes;
  }

  #toParents(): Map<string, string[]> {
    return Map<string, string[]>().withMutations((parents) => {
      const tasks = [this.getTask("root")];
      const visited = Set<string>().asMutable();
      while (tasks.length > 0) {
        const parent = tasks.shift()!;
        if (visited.contains(parent.id)) continue;
        visited.add(parent.id);
        for (const childId of parent.children) {
          const children = parents.get<string[]>(childId, []);
          children.push(parent.id);
          parents.set(childId, children);
          tasks.push(this.getTask(childId));
        }
      }
    });
  }

  // koso getters and setters

  get projectId(): string {
    return this.#projectId;
  }

  get doc(): Y.Doc {
    return this.#yDoc;
  }

  get clientId(): number {
    return this.doc.clientID;
  }

  get graph(): YGraphProxy {
    return this.#yGraph;
  }

  get tasks(): YTaskProxy[] {
    return this.#tasks;
  }

  get undoManager(): Y.UndoManager {
    return this.#yUndoManager;
  }

  /**
   * Returns the currently selected node, even if it no longer exists in the
   * nodes list.
   *
   * Most usages should prefer to use the `selected` getter below instead which
   * applies a filter to ensure the node exists.
   */
  get selectedRaw(): Selected {
    return this.#selectedRaw;
  }

  /**
   * Returns the currently selected node or null if none is selected.
   *
   * Note: this can change if, for example, a task is deleted from the graph
   * causing the selected node to no longer exist.
   */
  get selected(): Node | null {
    return this.#selected;
  }

  set selected(node: Node | null) {
    if (node && node.id === "root") {
      throw new Error("Cannot select root");
    }

    const shouldUpdateAwareness =
      !!this.#selectedRaw.node !== !!node ||
      (this.#selectedRaw.node && !this.#selectedRaw.node.equals(node));

    if (node) {
      // We need to expand the selected node's ancestors to ensure
      // the selected node is visible.
      this.expand(node.parent);

      const index = this.nodes.indexOf(node);
      if (index === -1) {
        // TODO: This happens when handleRow click is triggered when setting status to done in the inbox.
        // It'd be better if this threw.
        console.warn(`Selected node ${node.id} not found in nodes.`);
        return;
      }
      if (index === 0) {
        throw new Error(
          `Cannot selected root node ${node.id} at nodes index 0`,
        );
      }
      this.#selectedRaw = Selected.create(node, index);
      this.focus = true;
    } else {
      this.#selectedRaw = Selected.default();
    }

    if (shouldUpdateAwareness) {
      this.#send(this.#encodeAwareness());
    }
  }

  get awareness(): Awareness[] {
    return this.#awareness;
  }

  get focus(): boolean {
    return this.#focus;
  }

  set focus(value: boolean) {
    this.#focus = value;
  }

  get highlighted(): string | null {
    return this.#highlighted;
  }

  set highlighted(value: string | null) {
    this.#highlighted = value;
  }

  get dragged(): Node | null {
    return this.#dragged;
  }

  set dragged(value: Node | null) {
    this.#dragged = value;
  }

  get dropEffect(): "copy" | "move" | "none" {
    return this.#dropEffect;
  }

  set dropEffect(value: "copy" | "move" | "none") {
    this.#dropEffect = value;
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

  get synced(): Promise<[void, void]> {
    return Promise.all([this.#indexedDbSynced, this.#serverSynced]);
  }

  // composable functions that primarily operate on Tasks

  /** Converts the graph to JSON. */
  toJSON(): { [id: string]: Task } {
    return this.graph.toJSON();
  }

  /** Retrieves a task by task ID. */
  getTask(taskId: string): YTaskProxy {
    return this.graph.get(taskId);
  }

  /** Retrieves the parent tasks of the given task ID. */
  getParents(taskId: string): YTaskProxy[] {
    return this.getParentIds(taskId).map((parentId) => this.getTask(parentId));
  }

  /** Retrieves the parent task IDs of the given task ID. */
  getParentIds(taskId: string): string[] {
    const parents = this.#parents.get(taskId);
    if (!parents) throw new Error(`No parents entry found for ${taskId}`);
    return parents;
  }

  /** Retrieves the children of a given task ID. */
  getChildren(taskId: string): YChildrenProxy {
    return this.getTask(taskId).children;
  }

  /** Retrieves the number of child tasks for a given task ID. */
  getChildCount(taskId: string): number {
    return this.getChildren(taskId).length;
  }

  /** Checks if a given task is the parent of the given child. */
  hasChild(parent: string, child: string): boolean {
    return this.getChildren(parent).includes(child);
  }

  /**
   * Determines the status of a task. If the task has no children, the status is
   * derived directly from the task's status field. If the task has children,
   * the status is derived from the progress of its children.
   */
  getStatus(taskId: string): Status {
    return this.getProgress(taskId).status;
  }

  /**
   * Calculates the progress of a task. If the task has no children, the
   * progress is derived from the task's status field. If the task has children,
   * the progress is derived from the progress of its children.
   *
   * @param taskId - The unique identifier of the task.
   * @returns An object representing the progress of the task, including:
   *
   *   - `inProgress`: The number of tasks that are in progress.
   *   - `done`: The number of tasks that are done.
   *   - `total`: The total number of tasks.
   *   - `lastStatusTime`: The most recent status time of its children.
   *
   * @throws Will throw an error if the task status is invalid.
   */
  getProgress(taskId: string): Progress {
    const task = this.getTask(taskId);

    const result: Progress = new Progress({
      inProgress: 0,
      done: 0,
      total: 0,
      lastStatusTime: task.statusTime ?? 0,
      kind:
        task.yKind == "Juggled" && task.children.length === 0
          ? null
          : task.yKind || (task.children.length > 0 ? "Rollup" : null),
    });

    if (result.kind !== "Rollup") {
      switch (task.yStatus || "Not Started") {
        case "Done":
          result.done = 1;
          result.total = 1;
          result.status = "Done";
          break;
        case "In Progress":
          result.inProgress = 1;
          result.total = 1;
          result.status = "In Progress";
          break;
        case "Not Started":
          result.total = 1;
          result.status = "Not Started";
          break;
        case "Blocked":
          result.total = 1;
          result.status = "Blocked";
          break;
        default:
          throw new Error(
            `Invalid status ${task.yStatus} for task ${task.name}`,
          );
      }
    }

    let childInProgress = 0;
    let childDone = 0;
    let childTotal = 0;
    let childLastStatusTime = 0;
    task.children.forEach((taskId) => {
      // If performance is ever an issue for large, nested graphs,
      // we can memoize the recursive call and trade memory for time.
      const childProgress = this.getProgress(taskId);
      childInProgress += childProgress.inProgress;
      childDone += childProgress.done;
      childTotal += childProgress.total;
      childLastStatusTime = Math.max(
        childLastStatusTime,
        childProgress.lastStatusTime,
      );
    });
    result.lastStatusTime = Math.max(
      result.lastStatusTime,
      childLastStatusTime,
    );
    if (childTotal > 0) {
      if (childDone === childTotal) {
        result.childrenStatus = "Done";
      } else if (childInProgress > 0 || childDone > 0) {
        result.childrenStatus = "In Progress";
      } else {
        result.childrenStatus = "Not Started";
      }
    }

    if (result.kind === "Rollup") {
      result.inProgress += childInProgress;
      result.done += childDone;
      result.total += childTotal;
      result.status = result.childrenStatus || "Not Started";
    } else if (result.kind === "Juggled") {
      if (result.status === "Blocked" && !result.isChildrenIncomplete()) {
        // Auto-unblock unblocked tasks with the Blocked status
        result.status = "Not Started";
      }
    }

    return result;
  }

  /** Inserts or updates a task in the graph. */
  upsert(task: Task): YTaskProxy {
    return this.graph.set(task);
  }

  /**
   * Upserts the root task into the document. This method ensures that the root
   * task with a predefined structure is present in the document. After the root
   * task is upserted, the undo manager is cleared to prevent undoing the
   * creation of the root task.
   */
  upsertRoot() {
    this.doc.transact(() => {
      this.upsert({
        id: "root",
        num: "0",
        name: "Root",
        children: [],
        reporter: null,
        assignee: null,
        status: null,
        statusTime: null,
        kind: null,
        url: null,
      });
    }, "koso.upsertRoot");
    this.undoManager.clear();
  }

  select(taskId: string) {
    const nodes = this.getNodes(taskId);
    if (!nodes.length) throw new Error("Expected at least one Node");
    this.selected = nodes[0];
  }

  getNodes(taskId: string, slugs: List<string> = List()): Node[] {
    if (taskId === "root") {
      return [new Node({ path: slugs })];
    }
    slugs = slugs.insert(0, taskId);
    const nodes: Node[] = [];
    for (const parent of this.getParentIds(taskId)) {
      nodes.push(...this.getNodes(parent, slugs));
    }
    return nodes;
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

  getBestLinkOffset(taskId: string, parent: string): number {
    if (this.getStatus(taskId) === "In Progress") {
      return findEntryIndex(
        this.getChildren(parent).entries(),
        (peerId) => this.getStatus(peerId) !== "In Progress",
        0,
      );
    }
    return (
      findEntryIndex(
        this.getChildren(parent).entries({ step: -1 }),
        (peerId) => this.getStatus(peerId) !== "Done",
        this.getChildCount(parent) - 1,
      ) + 1
    );
  }

  /** Determines if a task can be linked to a parent task. */
  canLink(task: string, parent: string): boolean {
    return (
      !this.#hasCycle(parent, task) &&
      !this.hasChild(parent, task) &&
      this.isEditable(parent)
    );
  }

  /**
   * Links a task to a parent task. If the task introduces a cycle or is already
   * a child of the parent, an error is thrown. If an offset is not provided,
   * the best offset is determined based on the task's status.
   */
  link(task: string, parent: string, offset?: number) {
    if (!this.canLink(task, parent)) {
      throw new Error(`Cannot insert ${task} under ${parent}`);
    }

    offset = offset ?? this.getBestLinkOffset(task, parent);
    this.#linkUnchecked(task, parent, offset);
  }

  #linkUnchecked(task: string, parent: string, offset: number) {
    this.getChildren(parent).insert(offset, [task]);
  }

  canUnlink(task: string, parent: string): boolean {
    return !this.#isCanonicalManagedLink(task, parent);
  }

  /**
   * Unlinks a task from a parent task. Note that this method does not ensure
   * that the task is deleted from the graph and as a result may produce
   * orphaned tasks. To safely unlink a task, use {@link deleteNode} instead.
   *
   * @param taskId - The ID of the task to be unlinked.
   * @param parentId - The ID of the parent task.
   * @throws Will throw an error if the task is not found in the parent's
   *   children.
   */
  unlink(taskId: string, parentId: string) {
    if (!this.canUnlink(taskId, parentId)) {
      throw new Error(`Cannot unlink ${taskId} from parent ${parentId}`);
    }

    const parent = this.getTask(parentId);
    const index = parent.children.indexOf(taskId);
    if (index < 0)
      throw new Error(`Task ${taskId} is not in the children of ${parentId}`);
    parent.children.delete(index);
  }

  canMove(task: string, src: string, dest: string): boolean {
    return (
      src === dest ||
      (!this.#isCanonicalManagedLink(task, src) && this.canLink(task, dest))
    );
  }

  // business logic that operate on Nodes

  canExpand(node: Node) {
    return !this.expanded.contains(node) && this.getChildCount(node.name) > 0;
  }

  expand(node: Node) {
    while (node.name !== "root") {
      this.expanded = this.expanded.add(node);
      node = node.parent;
    }
  }

  canCollapse(node: Node) {
    return this.expanded.contains(node) && this.getChildCount(node.name) > 0;
  }

  collapse(node: Node) {
    const selected = this.selectedRaw;
    this.expanded = this.expanded.delete(node);
    if (selected.node && selected.node.isDescendantOf(node)) {
      this.selected = node;
    }
  }

  /**
   * Recursively accumulates all nodes with children and returns them in a
   * format that is suitable to be assigned to {@link expanded}.
   */
  #expandAll(
    node: Node = new Node(),
    accumulator: Set<Node> = Set(),
  ): Set<Node> {
    const task = this.getTask(node.name);
    if (task.children.length > 0) {
      accumulator = accumulator.add(node);
      task.children.forEach((name) => {
        const childNode = node.child(name);
        accumulator = this.#expandAll(childNode, accumulator);
      });
    }
    return accumulator;
  }

  /** Expands all tasks. */
  expandAll() {
    this.expanded = this.#expandAll();
  }

  /** Collapses all tasks. */
  collapseAll() {
    this.expanded = this.expanded.clear();
    this.selected = null;
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
    for (const peer of peers.slice({ start: prevPeerOffset, step: -1 })) {
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
    for (const peer of peers.slice({ start: nextPeerOffset })) {
      const peerNode = parent.child(peer);
      if (this.nodes.includes(peerNode)) {
        return peerNode;
      }
    }
    return null;
  }

  getPrevLink(node: Node): Node | null {
    const curr = this.nodes.indexOf(node);
    if (curr <= 0) {
      return null;
    }

    for (let i = curr - 1; i > 0; i--) {
      const n = this.#nodes.get(i);
      if (!n) throw new Error(`Node at ${i} does not exist.`);

      if (n.name === node.name) {
        return n;
      }
    }
    // Loop around from the end.
    for (let i = this.#nodes.size - 1; i > curr; i--) {
      const n = this.#nodes.get(i);
      if (!n) throw new Error(`Node at ${i} does not exist.`);

      if (n.name === node.name) {
        return n;
      }
    }

    return null;
  }

  getNextLink(node: Node): Node | null {
    const curr = this.nodes.indexOf(node);
    if (curr <= 0) {
      return null;
    }

    for (let i = curr + 1; i < this.nodes.size; i++) {
      const n = this.#nodes.get(i);
      if (!n) throw new Error(`Node at ${i} does not exist.`);

      if (n.name === node.name) {
        return n;
      }
    }
    // Loop around from the start
    for (let i = 1; i < curr; i++) {
      const n = this.#nodes.get(i);
      if (!n) throw new Error(`Node at ${i} does not exist.`);

      if (n.name === node.name) {
        return n;
      }
    }

    return null;
  }

  canDeleteNode(task: string, parent: string): boolean {
    return !this.#isCanonicalManagedLink(task, parent);
  }

  deleteNode(node: Node) {
    if (!this.canDeleteNode(node.name, node.parent.name)) {
      throw new Error(`Cannot delete node ${node}`);
    }

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

    this.doc.transact(() => {
      this.unlink(node.name, node.parent.name);
      for (const taskId of orphanTaskIds) {
        this.graph.delete(taskId);
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

  /**
   * Determines if the given task is a managed task, as indicated by the `kind`
   * property.
   */
  isManagedTask(taskId: string): boolean {
    const kind = this.getTask(taskId).yKind;
    return !!kind && !unmanagedKinds.includes(kind);
  }

  /**
   * Determines if a task is editable by users. Today, only tasks managed by a
   * plugin are not editable.
   */
  isEditable(taskId: string): boolean {
    return !this.isManagedTask(taskId);
  }

  /**
   * Determines if the given task is the canonical plugin task managed by a
   * plugin. As opposed to a link to the canonical task or container.
   *
   * The top-level plugin container is always a child of `root`. Plugin tasks
   * and containers are nested underneath. In the case of the github plugin, the
   * hierarchy is: `root -> github -> github_pr -> [some pr task]`
   *
   * Links to these in other locations are non-canonical and thus this method
   * would return false. `root -> github_pr` or `root -> [some pr task]`, for
   * example.
   */
  #isCanonicalManagedLink(task: string, parent: string): boolean {
    if (task === "root") {
      return true;
    }
    const kind = this.getTask(task).yKind;
    if (!kind || unmanagedKinds.includes(kind)) {
      return false;
    }
    // Is an immediate child of a plugin container OR is a plugin container.
    if (kind === parent) {
      return true;
    } else if (
      kind.startsWith(parent + "_") &&
      !kind.substring(parent.length + 1).includes("_")
    ) {
      return true;
    }
    // Is a top-level plugin container under root.
    // Underscores are used to separate hierarchical kinds.
    // Thus, if there is no underscore, the kind must be managed under root.
    if (!kind.includes("_") && parent === "root") {
      return true;
    }
    return false;
  }

  linkNode(node: Node, parent: Node, offset: number) {
    this.link(node.name, parent.name, offset);
  }

  canMoveNode(node: Node, parent: Node): boolean {
    return this.canMove(node.name, node.parent.name, parent.name);
  }

  moveNode(node: Node, parent: Node, offset: number) {
    if (offset < 0) {
      throw new Error(`Cannot move  ${node.name} to negative offset ${offset}`);
    }
    if (!this.canMoveNode(node, parent))
      throw new Error(`Cannot move ${node.name} to ${parent}`);
    const srcOffset = this.getOffset(node);
    this.doc.transact(() => {
      const srcParentName = node.parent.name;
      const ySrcChildren = this.getChildren(srcParentName);
      ySrcChildren.delete(srcOffset);

      if (srcParentName === parent.name && srcOffset < offset) {
        offset -= 1;
      }
      this.#linkUnchecked(node.name, parent.name, offset);
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
      if (!this.canMoveNode(node, newParent)) {
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
      if (!this.canMoveNode(node, newParent)) {
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

  moveNodeUpBoundary(node: Node) {
    const taskIds = this.getChildren(node.parent.name);
    const offset = taskIds.indexOf(node.name);

    if (offset === -1) {
      throw new Error(
        `Node ${node.name} not found in parent ${node.parent.id}`,
      );
    }
    if (offset === 0) {
      toast.warning(`This task is already at the top`);
      return;
    }

    const prev = this.getStatus(taskIds.get(offset - 1));
    for (const [index, taskId] of taskIds.entries({
      start: offset - 1,
      step: -1,
    })) {
      const curr = this.getStatus(taskId);
      if (curr !== prev) {
        this.reorderNode(node, index + 1);
        return;
      }
    }
    this.reorderNode(node, 0);
  }

  moveNodeDownBoundary(node: Node) {
    const taskIds = this.getChildren(node.parent.name);
    const offset = taskIds.indexOf(node.name);

    if (offset === -1) {
      throw new Error(
        `Node ${node.name} not found in parent ${node.parent.id}`,
      );
    }
    if (offset === taskIds.length - 1) {
      toast.warning(`This task is already at the bottom`);
      return;
    }

    const prev = this.getStatus(taskIds.get(offset + 1));
    for (const [index, taskId] of taskIds.entries({ start: offset + 1 })) {
      const curr = this.getStatus(taskId);
      if (curr !== prev) {
        this.reorderNode(node, index);
        return;
      }
    }
    this.reorderNode(node, taskIds.length);
  }

  canIndentNode(node: Node): boolean {
    const peer = this.getPrevPeer(node);
    return !!peer && this.canMoveNode(node, peer);
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
    return this.canMoveNode(node, node.parent.parent);
  }

  undentNode(node: Node) {
    if (!this.canUndentNode(node)) return;
    const parent = node.parent;
    const offset = this.getOffset(parent);
    this.moveNode(node, parent.parent, offset + 1);
    this.selected = parent.parent.child(node.name);
  }

  newId(): string {
    return uuidv4();
  }

  newNum(): string {
    let max = 0;
    for (const task of this.graph.values()) {
      const curr = parseInt(task.num);
      if (curr > max) {
        max = curr;
      }
    }
    return `${max + 1}`;
  }

  /** Determines whether the given task may have children inserted. */
  canInsert(parentTaskId: string): boolean {
    return this.isEditable(parentTaskId);
  }

  insertNode(
    parent: Node,
    offset: number,
    user: User,
    name: string = "",
  ): Node {
    if (!this.canInsert(parent.name)) {
      throw new Error(`Cannot insert node under parent ${parent}`);
    }
    const taskId = this.newId();
    this.doc.transact(() => {
      this.upsert({
        id: taskId,
        num: this.newNum(),
        name: name,
        children: [],
        reporter: user.email,
        assignee: null,
        status: null,
        statusTime: null,
        kind: null,
        url: null,
      });
      this.link(taskId, parent.name, offset);
    });
    const node = parent.child(taskId);
    this.selected = node;
    return node;
  }

  setTaskName(taskId: string, newName: string) {
    this.doc.transact(() => {
      const task = this.getTask(taskId);
      if (task.name !== newName) {
        task.name = newName;
      }
    });
  }

  setAssignee(taskId: string, assignee: User | null) {
    this.doc.transact(() => {
      const task = this.getTask(taskId);
      if (assignee === null && task.assignee !== null) {
        task.assignee = null;
      } else if (assignee && assignee.email !== task.assignee) {
        task.assignee = assignee.email;
      }
    });
  }

  setReporter(taskId: string, reporter: User | null) {
    this.doc.transact(() => {
      const task = this.getTask(taskId);
      if (reporter === null && task.reporter !== null) {
        task.reporter = null;
      } else if (reporter && reporter.email !== task.reporter) {
        task.reporter = reporter.email;
      }
    });
  }

  setKind(taskId: string, kind: Kind, user: User): boolean {
    return this.doc.transact(() => {
      const task = this.getTask(taskId);
      if (kind === "Juggled") {
        const progress = this.getProgress(taskId);
        if (progress.isChildrenIncomplete()) {
          task.yKind = "Juggled";
          task.yStatus = "Blocked";
          task.statusTime = Date.now();
          if (!task.assignee) {
            task.assignee = user.email;
          }
          toast.success(
            "Task is blocked. Koso Juggler will let you know when the task is unblocked! 🤹",
          );
          return true;
        } else {
          toast.info(
            "Task is immediately unblocked. Add a not done child first and then set the task to Blocked.",
          );
          return false;
        }
      } else if (kind === "Rollup") {
        task.yKind = null;
        task.yStatus = null;
        task.statusTime = Date.now();
        return true;
      } else {
        throw new Error(`Tried to set invalid kind: ${kind}`);
      }
    });
  }

  setTaskStatus(node: Node, status: Status, user: User): boolean {
    const taskId = node.name;
    return this.doc.transact(() => {
      const task = this.getTask(taskId);
      if (task.yStatus === status) return false;

      // When a task is marked done, make it the last child
      // and select an adjacent peer.
      if (status === "Done") {
        task.yStatus = status;
        task.statusTime = Date.now();

        return true;
      }
      // When a task is marked in progress, make it the first child
      // and, if unassigned, assign to the current user
      else if (status === "In Progress") {
        task.yStatus = status;
        task.statusTime = Date.now();

        if (!task.assignee) {
          task.assignee = user.email;
        }

        return true;
      } else if (status === "Blocked") {
        if (task.yKind !== "Juggled") {
          throw new Error(`Can only set Juggled tasks to blocked: ${taskId}`);
        }

        const progress = this.getProgress(taskId);
        if (progress.isChildrenIncomplete()) {
          task.yStatus = status;
          task.statusTime = Date.now();
          if (!task.assignee) {
            task.assignee = user.email;
          }
          toast.success(
            "Task is blocked. Koso Juggler will let you know when the task is unblocked! 🤹",
          );
          return true;
        } else {
          toast.info(
            "Task is immediately unblocked. Add a not done child first and then set the task to Blocked.",
          );
          return false;
        }
      } else {
        task.yStatus = status;
        task.statusTime = Date.now();
        return true;
      }
    });
  }

  isVisible(node: Node, showDone: boolean) {
    return this.#visibilityFilterFn(node, showDone);
  }

  /** Do not call this directly. Use isVisible instead. */
  #defaultIsVisible(node: Node) {
    if (!this.showDone) {
      const progress = this.getProgress(node.name);
      if (progress.isComplete()) {
        const doneTime = progress.lastStatusTime;
        const threeDays = 3 * 24 * 60 * 60 * 1000;
        return Date.now() - doneTime < threeDays;
      }
    }
    return true;
  }

  undo() {
    this.undoManager.undo();
  }

  redo() {
    this.undoManager.redo();
  }

  organizeTasks(node: Node) {
    function mapStatus(status: Status) {
      switch (status) {
        case "In Progress":
          return 0;
        case "Not Started":
          return 1;
        case "Blocked":
          return 2;
        case "Done":
          return 3;
        default:
          throw new Error(`Invalid status ${status}`);
      }
    }

    const parent = this.getTask(node.parent.name);
    // Sort tasks by status, otherwise
    // leaving the ordering unchanged thanks to sort() being stable.
    const children = parent.children
      .toArray()
      .map((taskId) => ({
        taskId,
        progress: this.getProgress(taskId),
      }))
      .sort((c1, c2) => {
        const status1 = mapStatus(c1.progress.status);
        const status2 = mapStatus(c2.progress.status);
        return status1 - status2;
      })
      .map((c) => c.taskId);

    this.doc.transact(() => {
      parent.children.set(children);
    });
  }
}
