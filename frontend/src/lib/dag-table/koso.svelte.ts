import { page } from "$app/state";
import { toast } from "$lib/components/ui/sonner";
import {
  parseAwarenessStateResponse,
  type Awareness,
  type AwarenessUpdate,
} from "$lib/dag-table/awareness.svelte";
import type { User } from "$lib/users";
import { findEntryIndex } from "$lib/utils";
import { Map, Record, Set } from "immutable";
import * as decoding from "lib0/decoding";
import * as encoding from "lib0/encoding";
import { v4 as uuidv4 } from "uuid";
import { IndexeddbPersistence } from "y-indexeddb";
import * as Y from "yjs";
import {
  YChildrenProxy,
  YGraphProxy,
  YTaskProxy,
  type Iteration,
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

type TaskLinkageProps = { id: string; parentId: string };
const TaskLinkageRecord = Record<TaskLinkageProps>({ parentId: "", id: "" });

export class TaskLinkage extends TaskLinkageRecord {
  toString(): string {
    return `${this.parentId}->${this.id}`;
  }
}

export class Progress {
  inProgress: number;
  done: number;
  total: number;
  lastStatusTime: number;
  status: Status;
  childrenStatus: Status | null;
  estimate: number | null;
  remainingEstimate: number | null;

  constructor(props: Partial<Progress> = {}) {
    this.inProgress = props.inProgress ?? 0;
    this.done = props.done ?? 0;
    this.total = props.total ?? 0;
    this.lastStatusTime = props.lastStatusTime ?? 0;
    this.status = props.status ?? "Not Started";
    this.childrenStatus = props.childrenStatus ?? null;
    this.estimate = props.estimate ?? null;
    this.remainingEstimate = props.remainingEstimate ?? null;
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

export class Koso {
  #projectId: string;
  #yDoc: Y.Doc;
  #yGraph: YGraphProxy;
  #yIndexedDb: IndexeddbPersistence;
  #send: (message: Uint8Array) => void = () => {
    // Until we connect to the server and invoke handleClientMessage,
    // there's nothing else to do with client messages, so we simply discard them.
    // Any dropped changes will be sync'd to the server later.
    console.debug("Client message handler was invoked but was not set");
  };

  #awareness: Awareness[] = $state([]);
  #awarenessSequence: number = 0;

  events: YEvent[] = $state.raw([]);
  #tasks: YTaskProxy[] = $derived.by(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    this.events;
    return Array.from(this.graph.values());
  });
  #parents: Map<string, string[]> = $derived.by(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    this.events;
    return this.#toParents();
  });

  #resolveIndexedDbSync: () => void = () => {};
  #indexedDbSynced = new Promise<void>(
    (resolve) => (this.#resolveIndexedDbSync = resolve),
  );
  #resolveServerSync: () => void = () => {};
  #serverSynced = new Promise<void>(
    (resolve) => (this.#resolveServerSync = resolve),
  );

  // lifecycle functions
  // i.e., init functions and helpers, event handlers, and destructors

  constructor(projectId: string, yDoc: Y.Doc) {
    this.#projectId = projectId;
    this.#yDoc = yDoc;
    const graph = yDoc.getMap<YTask>("graph");
    this.#yGraph = new YGraphProxy(graph);

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

    this.#yIndexedDb.whenSynced.then(() => {
      this.#resolveIndexedDbSync();
    });

    $effect(() => {
      const observer = (events: YEvent[]) => (this.events = events);

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
  }

  #encodeSyncRequest(): Uint8Array {
    const encoder = encoding.createEncoder();
    encoding.writeVarUint(encoder, MSG_SYNC);
    encoding.writeVarUint(encoder, MSG_SYNC_REQUEST);
    const sv = Y.encodeStateVector(this.doc);
    encoding.writeVarUint8Array(encoder, sv);
    return encoding.toUint8Array(encoder);
  }

  sendAwareness(selectedNodeId: string | null) {
    this.#send(this.#encodeAwareness(selectedNodeId));
  }

  #encodeAwareness(selectedNodeId: string | null): Uint8Array {
    const encoder = encoding.createEncoder();
    encoding.writeVarUint(encoder, MSG_KOSO_AWARENESS);
    encoding.writeVarUint(encoder, MSG_KOSO_AWARENESS_UPDATE);
    const update: AwarenessUpdate = {
      clientId: this.clientId,
      sequence: this.#awarenessSequence++,
      selected: selectedNodeId ? [selectedNodeId] : [],
    };
    encoding.writeVarString(encoder, JSON.stringify(update));
    return encoding.toUint8Array(encoder);
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

  get awareness(): Awareness[] {
    return this.#awareness;
  }

  get parents(): Map<string, string[]> {
    return this.#parents;
  }

  get synced(): Promise<void> {
    return Promise.any([this.#indexedDbSynced, this.#serverSynced]).then(() => {
      if (this.#yGraph.size > 0) {
        return;
      }
      // If the root node doesn't exist yet, wait for the server to sync before proceeding.
      // This avoids treating a project as sync'd on the first visit when indexedDB
      // hasn't yet cached the data.
      return Promise.all([this.#indexedDbSynced, this.#serverSynced]).then();
    });
  }

  get serverSynced(): Promise<void> {
    return this.#serverSynced;
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

  /**
   * Retrieves the index of the task in tasks {@link tasks}, if found, and -1
   * otherwise.
   */
  getTaskIndex(taskId: string): number {
    return this.#tasks.findIndex((t) => t.id === taskId);
  }

  /** Retrieves the parent tasks of the given task ID. */
  getParents(taskId: string): YTaskProxy[] {
    return this.getParentIds(taskId).map((parentId) => this.getTask(parentId));
  }

  /** Retrieves the parent task IDs of the given task ID. */
  getParentIds(taskId: string): string[] {
    return this.#parents.get(taskId) ?? [];
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

  /** Retrieves the offset, index, of the child in the given parent. */
  getChildTaskOffset(child: string, parent: string): number {
    const offset = this.getChildren(parent).indexOf(child);
    if (offset < 0) throw new Error(`Node ${child} not found in parent`);
    return offset;
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
    return this.#recursivelyGetProgress(taskId, {});
  }

  /** Helper for getProgress. Don't use directly. */
  #recursivelyGetProgress(
    taskId: string,
    visited: { [taskId: string]: boolean },
  ): Progress {
    visited[taskId] = true;

    const task = this.getTask(taskId);

    let childInProgress = 0;
    let childDone = 0;
    let childTotal = 0;
    let childLastStatusTime = 0;
    let childrenEstimate: number | null = null;
    let childrenRemainingEstimate: number | null = null;
    task.children.forEach((taskId) => {
      // Avoid re-counting tasks present more than once in a sub-tree.
      if (visited[taskId]) {
        return;
      }

      // If performance is ever an issue for large, nested graphs,
      // we can memoize the recursive call and trade memory for time.
      const childProgress = this.#recursivelyGetProgress(
        taskId,
        // We only need to dedupe for rollups.
        // Child estimates and counts are NOT carried forward for other types.
        task.isRollup() ? visited : {},
      );
      childInProgress += childProgress.inProgress;
      childDone += childProgress.done;
      childTotal += childProgress.total;
      childLastStatusTime = Math.max(
        childLastStatusTime,
        childProgress.lastStatusTime,
      );
      if (childProgress.estimate !== null) {
        childrenEstimate = (childrenEstimate ?? 0) + childProgress.estimate;
      }
      if (childProgress.remainingEstimate !== null) {
        childrenRemainingEstimate =
          (childrenRemainingEstimate ?? 0) + childProgress.remainingEstimate;
      }
    });

    let childrenStatus: Status | null;
    if (childTotal > 0) {
      if (childDone === childTotal) {
        childrenStatus = "Done";
      } else if (childInProgress > 0 || childDone > 0) {
        childrenStatus = "In Progress";
      } else {
        childrenStatus = "Not Started";
      }
    } else {
      childrenStatus = null;
    }

    if (task.isRollup()) {
      return new Progress({
        inProgress: childInProgress,
        done: childDone,
        total: childTotal,
        status: childrenStatus ?? (childTotal === 0 ? "Done" : "Not Started"),
        lastStatusTime: Math.max(task.statusTime ?? 0, childLastStatusTime),
        estimate: childrenEstimate,
        remainingEstimate: childrenRemainingEstimate,
        childrenStatus,
      });
    } else {
      let status = task.yStatus || "Not Started";
      // Auto-unblock unblocked tasks with the Blocked status
      if (status === "Blocked") {
        const childrenComplete =
          childrenStatus === null || childrenStatus === "Done";
        if (childrenComplete) {
          status = "Not Started";
        }
      }

      return new Progress({
        inProgress: status === "In Progress" ? 1 : 0,
        done: status === "Done" ? 1 : 0,
        total: 1,
        status,
        lastStatusTime: Math.max(task.statusTime ?? 0, childLastStatusTime),
        estimate: task.estimate,
        remainingEstimate:
          task.estimate === null ? null : status === "Done" ? 0 : task.estimate,
        childrenStatus,
      });
    }
  }

  getIterations(): Iteration[] {
    return this.tasks
      .filter((task) => task.isIteration())
      .sort((taskA, taskB) => taskA.deadline - taskB.deadline);
  }

  getCurrentIterations(): Iteration[] {
    return this.getIterations().filter((task) => task.deadline > Date.now());
  }

  hasDescendant(ancestor: string, descendant: string): boolean {
    const parents = [descendant];

    while (parents.length > 0) {
      const curr = parents.pop()!;
      if (ancestor === curr) return true;
      parents.push(...this.getParentIds(curr));
    }

    return false;
  }

  /** Inserts or updates a task in the graph. */
  upsert(task: Task): YTaskProxy {
    return this.graph.set(task);
  }

  /**
   * Upserts the root task into the document. This method ensures that the root
   * task with a predefined structure is present in the document.
   */
  upsertRoot() {
    this.doc.transact(() => {
      this.upsert({
        id: "root",
        num: "0",
        name: "Root",
        desc: null,
        children: [],
        reporter: null,
        assignee: null,
        status: null,
        statusTime: null,
        kind: null,
        url: null,
        estimate: null,
        deadline: null,
        archived: null,
      });
    }, "koso.upsertRoot");
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

  getBestLinkOffset({ id: taskId, parentId: parent }: TaskLinkage): number {
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
  canLink({ id: task, parentId: parent }: TaskLinkage): boolean {
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
  link(linkage: TaskLinkage, offset?: number) {
    if (!this.canLink(linkage)) {
      throw new Error(`Cannot insert link ${linkage}`);
    }

    offset = offset ?? this.getBestLinkOffset(linkage);
    this.#linkUnchecked(linkage, offset);
  }

  /** Like {@link link} but with out the safety checks. Use cautiously. */
  #linkUnchecked({ id: task, parentId: parent }: TaskLinkage, offset: number) {
    this.getChildren(parent).insert(offset, [task]);
  }

  /** Determines if a task can be unlinked from a parent task. */
  canUnlink(task: string, parent: string): boolean {
    return !this.isCanonicalManagedLink(task, parent);
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
    console.debug(`Unlinking task ${taskId} from parent ${parentId}`);
    parent.children.delete(index);
  }

  canMove(task: string, src: string, dest: string): boolean {
    return (
      src === dest ||
      (!this.isCanonicalManagedLink(task, src) &&
        this.canLink(new TaskLinkage({ parentId: dest, id: task })))
    );
  }

  /** Moves the given task from one parent to another. */
  move(
    srcId: string,
    srcParentId: string,
    destParentId: string,
    offset: number,
  ) {
    if (offset < 0) {
      throw new Error(`Cannot move  ${srcId} to negative offset ${offset}`);
    }
    if (!this.canMove(srcId, srcParentId, destParentId))
      throw new Error(`Cannot move ${srcId} to ${destParentId}`);
    const srcOffset = this.getChildTaskOffset(srcId, srcParentId);
    this.doc.transact(() => {
      const ySrcChildren = this.getChildren(srcParentId);
      ySrcChildren.delete(srcOffset);

      if (srcParentId === destParentId && srcOffset < offset) {
        offset -= 1;
      }
      this.#linkUnchecked(
        new TaskLinkage({ parentId: destParentId, id: srcId }),
        offset,
      );
    });
  }

  canDelete(link: TaskLinkage): boolean {
    return !this.isCanonicalManagedLink(link.id, link.parentId);
  }

  canDeleteTask(id: string): boolean {
    const parentIds = this.parents.get(id);
    if (!parentIds) {
      return true;
    }
    return parentIds.every((parentId) =>
      this.canDelete(new TaskLinkage({ id, parentId })),
    );
  }

  delete(link: TaskLinkage) {
    if (!this.canDelete(link)) {
      throw new Error(`Cannot delete task ${link}`);
    }

    const subtreeTaskIds = this.#collectSubtreeTaskIds(link.id);

    // Find all of the tasks that will become orphans when the task
    // is unlinked. In other words, tasks whose only parents are also in the sub-tree
    // being deleted.
    const stack = [link.id];
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
        const isTargetTaskLink =
          taskId === link.id && parentTaskId === link.parentId;
        const parentInSubtree = subtreeTaskIds.has(parentTaskId);
        return !isTargetTaskLink && !parentInSubtree;
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
      this.unlink(link.id, link.parentId);
      for (const taskId of orphanTaskIds) {
        console.debug(`Deleting task: ${taskId}`);
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

  /** Deletes the given task and any orphaned descendants that result. */
  deleteTask(id: string) {
    if (!this.canDeleteTask(id)) {
      throw new Error(`Cannot delete task ${id}`);
    }

    const subtreeTaskIds = this.#collectSubtreeTaskIds(id);

    // Find all of the tasks that will become orphans when the task
    // is deleted. In other words, tasks whose only parents are also in the sub-tree
    // being deleted.
    const stack = [id];
    let orphanTaskIds = Set<string>();
    let visited = Set<string>();
    while (stack.length > 0) {
      const taskId = stack.pop();
      if (!taskId || visited.has(taskId)) {
        continue;
      }
      visited = visited.add(taskId);

      // Don't delete tasks that are linked to outside of the target sub-tree.
      const parents = this.parents.get(taskId) || [];
      const linkedElseWhere = parents.some((parentTaskId) => {
        const isTargetTaskLink = taskId === id;
        const parentInSubtree = subtreeTaskIds.has(parentTaskId);
        return !isTargetTaskLink && !parentInSubtree;
      });
      if (linkedElseWhere) {
        continue;
      }

      orphanTaskIds = orphanTaskIds.add(taskId);
      for (const childTaskId of this.getChildren(taskId)) {
        stack.push(childTaskId);
      }
    }

    const parentIds = this.parents.get(id) || [];
    this.doc.transact(() => {
      for (const parentId of parentIds) {
        this.unlink(id, parentId);
      }
      for (const taskId of orphanTaskIds) {
        console.debug(`Deleting task: ${taskId}`);
        this.graph.delete(taskId);
      }
    });
  }

  /**
   * Determines if the given task is a managed task, as indicated by the `kind`
   * property.
   */
  isManagedTask(taskId: string): boolean {
    return this.getTask(taskId).isManaged();
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
  isCanonicalManagedLink(taskId: string, parent: string): boolean {
    if (taskId === "root") {
      return true;
    }
    const task = this.getTask(taskId);
    if (!task.isManaged()) {
      return false;
    }
    const kind = task.kind;
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

  insertTask({
    name = "",
    parent,
    offset = 0,
    reporter,
    assignee = null,
  }: {
    name?: string;
    parent: string;
    offset?: number;
    reporter: string;
    assignee?: string | null;
  }): string {
    if (!this.canInsert(parent)) {
      throw new Error(`Cannot insert task under parent ${parent}`);
    }
    const taskId = this.newId();
    this.doc.transact(() => {
      this.upsert({
        id: taskId,
        num: this.newNum(),
        name,
        desc: null,
        children: [],
        reporter,
        assignee,
        status: null,
        statusTime: null,
        kind: null,
        url: null,
        estimate: null,
        deadline: null,
        archived: null,
      });
      this.link(new TaskLinkage({ parentId: parent, id: taskId }), offset);
    });
    return taskId;
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

  setKind(taskId: string, kind: Kind | null): boolean {
    return this.doc.transact(() => {
      const task = this.getTask(taskId);
      if (task.kind === kind) return false;

      if (
        kind === "Task" ||
        // Handle the transition to auto-task as well.
        (kind === null && task.autoType() === "Task")
      ) {
        const progress = this.getProgress(taskId);
        task.kind = kind;
        if (progress.status !== task.yStatus) {
          task.yStatus = progress.status;
          task.statusTime = Date.now();
        }
        return true;
      } else if (
        kind === "Rollup" ||
        // Handle the transition to auto-rollup as well.
        (kind === null && task.autoType() === "Rollup")
      ) {
        task.kind = kind;
        task.yStatus = null;
        task.statusTime = Date.now();
        return true;
      } else {
        throw new Error(`Tried to set invalid kind: ${kind}`);
      }
    });
  }

  setTaskStatus(taskId: string, status: Status, user: User): boolean {
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
        // TODO
        if (task.kind !== "Task") {
          throw new Error(`Can only set Tasks to blocked: ${taskId}`);
        }

        const progress = this.getProgress(taskId);
        if (progress.isChildrenIncomplete()) {
          task.yStatus = status;
          task.statusTime = Date.now();
          if (!task.assignee) {
            task.assignee = user.email;
          }
          toast.success(
            "Task is blocked. Koso will let you know when the task is unblocked! 🤹",
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

  toggleStatus(taskId: string, user: User) {
    const task = this.getTask(taskId);
    if (task.isRollup()) {
      toast.warning(
        "Cannot change the status of a Rollup task. Change the status of the task's children instead.",
      );
      return;
    }

    const progress = this.getProgress(task.id);
    switch (progress.status) {
      case "Done":
        return;
      case "Blocked":
        this.setTaskStatus(task.id, "Not Started", user);
        return;
      case "In Progress": {
        this.setTaskStatus(task.id, "Done", user);
        toast.success("🚀 Great work! Task complete!");
        break;
      }
      case "Ready":
        this.setTaskStatus(task.id, "In Progress", user);
        break;
      case "Not Started":
        this.setTaskStatus(task.id, "Ready", user);
        break;
      default:
        throw new Error(`Unhandled status ${task.yStatus}`);
    }
  }

  setTaskArchived(taskId: string, archived: boolean) {
    const task = this.getTask(taskId);
    if (!!task.archived !== archived) {
      task.archived = archived;
    }
  }

  getTaskPermalink(taskId: string) {
    const curr = page.url;
    curr.pathname = `/projects/${this.projectId}`;
    curr.search = new URLSearchParams({ taskId }).toString();
    return curr;
  }

  getGitCommitId(taskId: string) {
    const task = this.getTask(taskId);
    return `koso-${task.num}`;
  }

  /** Organizes the given task's children by status, etc. */
  organizeTasks(parentTaskId: string) {
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

    const parent = this.getTask(parentTaskId);
    // Sort tasks by status, otherwise
    // leaving the ordering unchanged thanks to sort() being stable.
    const children = parent.children.toArray().map((taskId) => ({
      taskId,
      progress: this.getProgress(taskId),
    }));

    this.doc.transact(() => {
      // Archive any tasks that have been Done for awhile.
      const now = Date.now();
      children.forEach(({ taskId, progress }) => {
        if (progress.status === "Done") {
          const fourteenDays = 14 * 24 * 60 * 60 * 1000;
          const stale = now - progress.lastStatusTime > fourteenDays;
          if (stale) {
            const task = this.getTask(taskId);
            if (!task.archived) {
              console.log("Archiving");
              task.archived = true;
            }
          }
        }
      });

      // It's important to sort after archiving because sorting
      // depends on a tasks' archived state.
      const sortedChildren = children
        .map(({ taskId, progress }) => {
          return {
            taskId,
            progress,
            archived: this.getTask(taskId).archived,
          };
        })
        .sort((c1, c2) => {
          // Order non-archived tasks ahead of archived ones.
          if (!!c1.archived !== !!c2.archived) {
            return c1.archived ? 1 : -1;
          }
          const status1 = mapStatus(c1.progress.status);
          const status2 = mapStatus(c2.progress.status);
          return status1 - status2;
        })
        .map(({ taskId }) => taskId);
      parent.children.replace(sortedChildren);
    });
  }
}
