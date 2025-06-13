import { toast } from "$lib/components/ui/sonner";
import { Koso, TaskLinkage } from "$lib/dag-table/koso.svelte";
import { useLocalStorage, type Storable } from "$lib/stores.svelte";
import { List, Record, Set } from "immutable";
import { getContext, setContext } from "svelte";
import * as Y from "yjs";

export class PlanningContext {
  #koso: Koso;
  #root: Node;

  #yUndoManager: Y.UndoManager;

  #nodes: List<Node> = $derived.by(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    this.koso.events;
    // The nodes store is consistently initialized prior to the ygraph
    // being loaded, but #flatten expects the presence of at least a
    // "root" task. Handle the situation here to avoid generating warnings
    // in #flatten.
    if (this.koso.graph.size === 0) {
      return List();
    }
    return this.#defaultFlatten(this.root, this.expanded, this.showArchived);
  });

  #selectedRaw: Selected = $state(Selected.default());
  #selected: Node | null = $derived.by(() => {
    const node = this.#selectedRaw.node;
    if (!node || this.nodes.indexOf(node) < 0) {
      return null;
    }
    return node;
  });

  #expanded: Storable<Set<Node>>;
  #showArchived: Storable<boolean>;
  #highlighted: string | null = $state(null);
  #dragged: Node | null = $state(null);
  #dropEffect: "copy" | "move" | "none" = $state("none");
  #focus: boolean = $state(false);

  constructor(koso: Koso, root?: string) {
    this.#koso = koso;
    this.#root = root ? Node.parse(root) : new Node();

    this.#yUndoManager = new Y.UndoManager(this.koso.graph.yGraph, {
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

    this.#showArchived = useLocalStorage<boolean>(
      `show-done-${this.koso.projectId}`,
      false,
    );

    this.#expanded = useLocalStorage<Set<Node>>(
      `expanded-nodes-${this.koso.projectId}`,
      Set(),
      {
        decode: (json: string) => Set(JSON.parse(json).map(Node.parse)),
        encode: (nodes) => JSON.stringify(nodes.map((node) => node.id)),
      },
    );
  }

  get koso(): Koso {
    return this.#koso;
  }

  get undoManager(): Y.UndoManager {
    return this.#yUndoManager;
  }

  get root(): Node {
    return this.#root;
  }

  get nodes(): List<Node> {
    return this.#nodes;
  }

  get expanded(): Set<Node> {
    return this.#expanded.value;
  }

  set expanded(value: Set<Node>) {
    this.#expanded.value = value;
  }

  get showArchived(): boolean {
    return this.#showArchived.value;
  }

  set showArchived(value: boolean) {
    this.#showArchived.value = value;
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
      this.koso.sendAwareness(this.selected ? this.selected.id : null);
    }
  }

  // business logic that operate on Nodes

  select(taskId: string) {
    const nodes = this.getNodes(taskId);
    if (!nodes.length) throw new Error("Expected at least one Node");
    this.selected = nodes[0];
  }

  canExpand(node: Node) {
    return (
      !this.expanded.contains(node) && this.koso.getChildCount(node.name) > 0
    );
  }

  expand(node: Node) {
    while (node.name !== "root") {
      this.expanded = this.expanded.add(node);
      node = node.parent;
    }
  }

  canCollapse(node: Node) {
    return (
      this.expanded.contains(node) && this.koso.getChildCount(node.name) > 0
    );
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
  #expandAll(node: Node, accumulator: Set<Node> = Set()): Set<Node> {
    const task = this.koso.getTask(node.name);
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
    this.expanded = this.#expandAll(this.root);
  }

  /** Collapses all tasks. */
  collapseAll() {
    this.expanded = this.expanded.clear();
    this.selected = null;
  }

  getNodes(taskId: string, slugs: List<string> = List()): Node[] {
    if (taskId === "root") {
      return [new Node({ path: slugs })];
    }
    slugs = slugs.insert(0, taskId);
    const nodes: Node[] = [];
    for (const parent of this.#koso.getParentIds(taskId)) {
      nodes.push(...this.getNodes(parent, slugs));
    }
    return nodes;
  }

  getOffset(node: Node): number {
    return this.koso.getChildTaskOffset(node.name, node.parent.name);
  }

  getPrevPeer(node: Node): Node | null {
    const parent = node.parent;
    const peers = this.koso.getChildren(parent.name);
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
    const peers = this.koso.getChildren(parent.name);
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

  reorderNode(node: Node, offset: number) {
    this.moveNode(node, node.parent, offset);
  }

  canMoveNode(node: Node, parent: Node): boolean {
    return this.koso.canMove(node.name, node.parent.name, parent.name);
  }

  moveNode(node: Node, parent: Node, offset: number) {
    this.koso.move(node.name, node.parent.name, parent.name, offset);
    this.selected = parent.child(node.name);
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
      console.trace(
        `Trying to move up: newParent: ${newParent.id}, offset: ${newOffset}`,
      );
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
      console.trace(
        `Trying to move down: newParent: ${newParent.id}, offset: ${newOffset}`,
      );
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
    const taskIds = this.koso.getChildren(node.parent.name);
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

    const prev = this.koso.getStatus(taskIds.get(offset - 1));
    for (const [index, taskId] of taskIds.entries({
      start: offset - 1,
      step: -1,
    })) {
      const curr = this.koso.getStatus(taskId);
      if (curr !== prev) {
        this.reorderNode(node, index + 1);
        return;
      }
    }
    this.reorderNode(node, 0);
  }

  moveNodeDownBoundary(node: Node) {
    const taskIds = this.koso.getChildren(node.parent.name);
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

    const prev = this.koso.getStatus(taskIds.get(offset + 1));
    for (const [index, taskId] of taskIds.entries({ start: offset + 1 })) {
      const curr = this.koso.getStatus(taskId);
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
    this.moveNode(node, peer, this.koso.getChildCount(peer.name));
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

  linkNode(node: Node, parent: Node, offset: number) {
    this.koso.link(
      new TaskLinkage({ parentId: parent.name, id: node.name }),
      offset,
    );
  }

  #defaultFlatten(
    node: Node,
    expanded: Set<Node>,
    showArchived: boolean,
    nodes: List<Node> = List(),
  ): List<Node> {
    const task = this.koso.getTask(node.name);
    nodes = nodes.push(node);
    if (node.length < 1 || expanded.has(node)) {
      task.children.forEach((name) => {
        const childNode = node.child(name);
        // Apply visibility filtering here instead of at the start of #flatten
        // to ensure that the root node is always present.
        if (this.isVisible(childNode.name, showArchived)) {
          nodes = this.#defaultFlatten(
            childNode,
            expanded,
            showArchived,
            nodes,
          );
        }
      });
    }
    return nodes;
  }

  isVisible(taskId: string, showArchived: boolean) {
    return showArchived || !this.#koso.getTask(taskId).archived;
  }

  undo() {
    this.undoManager.undo();
  }

  redo() {
    this.undoManager.redo();
  }
}

export function newPlanningContext(koso: Koso, root?: string): PlanningContext {
  const ctx = new PlanningContext(koso, root);
  window.planningCtx = ctx;
  return setPlanningContext(ctx);
}

export function setPlanningContext(ctx: PlanningContext): PlanningContext {
  return setContext<PlanningContext>(PlanningContext, ctx);
}

export function getPlanningContext(): PlanningContext {
  const ctx = getContext<PlanningContext>(PlanningContext);
  if (!ctx) throw new Error("PlanningContext is undefined");
  return ctx;
}

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

  get linkage(): TaskLinkage {
    return new TaskLinkage({ parentId: this.parent.name, id: this.name });
  }
}

export type Nodes = Map<string, Node>;
