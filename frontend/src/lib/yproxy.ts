import { Set as ImmutableSet } from "immutable";
import * as Y from "yjs";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type YEvent = Y.YEvent<any>;
export type YGraph = Y.Map<YTask>;
export type YTask = Y.Map<YTaskProps>;
export type YTaskProps = YChildren | Y.Text | string | number | boolean | null;
export type YChildren = Y.Array<string>;

export type Graph = { [id: string]: Task };
export type Task = {
  id: string;
  num: string;
  name: string;
  desc: string | null;
  children: string[];
  assignee: string | null;
  reporter: string | null;
  status: Status | null;
  // Time, in milliseconds since the unix epoch,
  // when the `status` field was last modified.
  statusTime: number | null;
  // The kind of task, either directly associated with Koso
  // or a plugin. e.g. github.
  kind: Kind | null;
  // URL associated with the task. Typically provided by a plugin
  // to associate this task with some external entity.
  // e.g. a Github PR URL.
  url: string | null;
  // An estimate of how long the task will take to complete.
  estimate: Estimate | null;
  // When this task is targetted for completion.
  deadline: number | null;
  archived: boolean | null;
};
export type Status =
  | "Not Started"
  | "Ready"
  | "In Progress"
  | "Done"
  | "Blocked";
export type Kind = "Rollup" | "Task" | "github" | "github_pr";
// Keep this in sync with the corresponding list in
// backend/yproxy.rs
export const MANAGED_KINDS: ImmutableSet<Kind> = ImmutableSet.of(
  "github",
  "github_pr",
);
export const ESTIMATES = <const>[1, 2, 3, 5, 8, 13, 20];
export type Estimate = (typeof ESTIMATES)[number];

export type Slice = {
  start?: number;
  end?: number;
  step?: number;
};

export function defaultTask(): Omit<Task, "id" | "num" | "name"> {
  return {
    desc: null,
    children: [],
    assignee: null,
    reporter: null,
    status: null,
    statusTime: null,
    kind: null,
    url: null,
    estimate: null,
    deadline: null,
    archived: null,
  };
}

export class YGraphProxy {
  #yGraph: YGraph;

  constructor(yGraph: YGraph) {
    this.#yGraph = yGraph;
  }

  get size() {
    return this.#yGraph.size;
  }

  get yGraph() {
    return this.#yGraph;
  }

  keys(): IterableIterator<string> {
    return this.#yGraph.keys();
  }

  *values(): IterableIterator<YTaskProxy> {
    for (const taskId of this.keys()) {
      yield this.get(taskId);
    }
  }

  delete(taskId: string) {
    if (taskId === "root") {
      throw new Error("Cannot delete root node");
    }
    this.#yGraph.delete(taskId);
  }

  set(task: Task): YTaskProxy {
    const value = new Y.Map<YTaskProps>([
      ["id", task.id],
      ["num", task.num],
      ["name", task.name],
      ["desc", task.desc !== null ? new Y.Text(task.desc) : null],
      ["children", Y.Array.from(task.children)],
      ["reporter", task.reporter],
      ["assignee", task.assignee],
      ["status", task.status],
      ["statusTime", task.statusTime],
      ["kind", task.kind],
      ["url", task.url],
      ["estimate", task.estimate],
      ["deadline", task.deadline],
      ["archived", task.archived],
    ]);
    this.#yGraph.set(task.id, value);
    return new YTaskProxy(value);
  }

  has(taskId: string): boolean {
    return this.#yGraph.has(taskId);
  }

  get(taskId: string): YTaskProxy {
    const yTask = this.#yGraph.get(taskId);
    if (!yTask) throw new Error(`Unknown Task ID: ${taskId}`);
    return new YTaskProxy(yTask);
  }

  toJSON(): Graph {
    return this.#yGraph.toJSON();
  }

  observe(f: (arg0: YEvent[], arg1: Y.Transaction) => void) {
    this.#yGraph.observeDeep(f);
  }

  unobserve(f: (arg0: YEvent[], arg1: Y.Transaction) => void) {
    this.#yGraph.unobserveDeep(f);
  }
}

export type Iteration = YTaskProxy & { deadline: number };
export type Managed = YTaskProxy & { kind: Kind };

export class YTaskProxy {
  #yTask: YTask;
  #subscribers = new Set<(value: YTaskProxy) => void>();

  constructor(yTask: YTask) {
    this.#yTask = yTask;
  }

  get id(): string {
    return this.#yTask.get("id") as string;
  }

  get num(): string {
    return this.#yTask.get("num") as string;
  }

  set num(value: string) {
    this.#yTask.set("num", value);
  }

  get name(): string {
    return this.#yTask.get("name") as string;
  }

  set name(value: string) {
    this.#yTask.set("name", value);
  }

  get desc(): Y.Text | null {
    return (this.#yTask.get("desc") as Y.Text) || null;
  }

  newDesc() {
    if (this.desc) return;
    const desc = new Y.Text();
    this.#yTask.set("desc", desc);
  }

  delDesc() {
    this.#yTask.set("desc", null);
  }

  get children(): YChildrenProxy {
    const yChildren = this.#yTask.get("children") as YChildren;
    if (!yChildren) throw new Error("yChildren is undefined");
    return new YChildrenProxy(yChildren);
  }

  get assignee(): string | null {
    return (this.#yTask.get("assignee") as string) || null;
  }

  set assignee(value: string | null) {
    this.#yTask.set("assignee", value);
  }

  get reporter(): string | null {
    return (this.#yTask.get("reporter") as string) || null;
  }

  set reporter(value: string | null) {
    this.#yTask.set("reporter", value);
  }

  get yStatus(): Status | null {
    return (this.#yTask.get("status") as Status) || null;
  }

  set yStatus(value: Status | null) {
    this.#yTask.set("status", value);
  }

  get statusTime(): number | null {
    return (this.#yTask.get("statusTime") as number) || null;
  }

  set statusTime(value: number | null) {
    this.#yTask.set("statusTime", value);
  }

  get kind(): Kind | null {
    return this.#yTask.get("kind") as Kind;
  }

  set kind(value: Kind | null) {
    this.#yTask.set("kind", value);
  }

  get url(): string | null {
    return (this.#yTask.get("url") as string) || null;
  }

  get estimate(): Estimate | null {
    return (this.#yTask.get("estimate") as Estimate) ?? null;
  }

  set estimate(value: Estimate | null) {
    this.#yTask.set("estimate", value);
  }

  get deadline(): number | null {
    return (this.#yTask.get("deadline") as number) || null;
  }

  set deadline(value: number | null) {
    this.#yTask.set("deadline", value);
  }

  get archived(): boolean | null {
    const archived = this.#yTask.get("archived") as boolean;
    return archived ?? null;
  }

  set archived(value: boolean | null) {
    this.#yTask.set("archived", value);
  }

  isLeaf(): boolean {
    return this.children.length === 0;
  }

  isAuto(): boolean {
    return this.kind === null;
  }

  isTask(): boolean {
    return this.kind === "Task" || (this.isAuto() && this.isLeaf());
  }

  /**
   * Use in conjuction with {@link isAuto()} to identify auto tasks and their
   * type.
   */
  autoType(): "Rollup" | "Task" {
    return this.children.length > 0 ? "Rollup" : "Task";
  }

  /**
   * Determines whether this task is a Rollup type task.
   *
   * Note: iteration tasks are also rollups, but not all rollups are iterations.
   */
  isRollup(): boolean {
    const kind = this.kind;
    return kind === "Rollup" || (this.isAuto() && !this.isLeaf());
  }

  isIteration(): this is Iteration {
    return this.isRollup() && !!this.deadline;
  }

  isManaged(): this is Managed {
    return this.kind !== null && MANAGED_KINDS.contains(this.kind);
  }

  observe(f: (arg0: Y.YMapEvent<YTaskProps>, arg1: Y.Transaction) => void) {
    this.#yTask.observe(f);
    return () => this.unobserve(f);
  }

  unobserve(f: (arg0: Y.YMapEvent<YTaskProps>, arg1: Y.Transaction) => void) {
    this.#yTask.unobserve(f);
  }

  observeDeep(f: (arg0: YEvent[], arg1: Y.Transaction) => void) {
    this.#yTask.observeDeep(f);
    return () => this.unobserveDeep(f);
  }

  unobserveDeep(f: (arg0: YEvent[], arg1: Y.Transaction) => void) {
    this.#yTask.unobserveDeep(f);
  }

  #broadcast = () => {
    for (const subscriber of this.#subscribers) {
      subscriber(this);
    }
  };

  subscribe(subscriber: (value: YTaskProxy) => void): () => void {
    if (this.#subscribers.size === 0) {
      this.observeDeep(this.#broadcast);
    }

    this.#subscribers.add(subscriber);
    subscriber(this);

    return () => {
      this.#subscribers.delete(subscriber);
      if (this.#subscribers.size === 0) {
        this.unobserveDeep(this.#broadcast);
      }
    };
  }

  toJSON(): Task {
    return this.#yTask.toJSON() as Task;
  }
}

export class YChildrenProxy {
  #yChildren: YChildren;

  constructor(yChildren: YChildren) {
    this.#yChildren = yChildren;
  }

  get length(): number {
    return this.#yChildren.length;
  }

  *[Symbol.iterator]() {
    yield* this.#yChildren;
  }

  get(index: number): string {
    return this.#yChildren.get(index);
  }

  *slice(slice: Slice = {}): IterableIterator<string> {
    for (const entry of this.entries(slice)) {
      yield entry[1];
    }
  }

  *entries(slice: Slice = {}): IterableIterator<[number, string]> {
    const step = slice.step ?? 1;
    if (step === 0) throw new Error("Step size should not be zero");

    let start = slice.start;
    if (start === null || start === undefined) {
      start = step > 0 ? 0 : this.length - 1;
    } else if (start < 0) {
      start = Math.max(start + this.length, 0);
    }

    let end = slice.end;
    if (end === null || end === undefined) {
      end = step > 0 ? this.length : -1;
    } else if (end < 0) {
      end += this.length;
    } else if (end > this.length) {
      end = this.length;
    }

    for (
      let i = start;
      (step > 0 && i < end) || (step < 0 && i > end);
      i += step
    ) {
      yield [i, this.get(i)];
    }
  }

  insert(index: number, content: string[]) {
    this.#yChildren.insert(index, content);
  }

  push(content: string[]) {
    this.#yChildren.push(content);
  }

  delete(index: number, length?: number) {
    this.#yChildren.delete(index, length);
  }

  /**
   * Replaces the current children with the given ones.
   *
   * If the new children match the existing children, no changes are performed.
   */
  replace(content: string[]) {
    if (
      this.#yChildren.length === content.length &&
      this.#yChildren.toArray().every((v, i) => v === content[i])
    ) {
      return;
    }
    // TODO: Improve this with a clever diff. a la Rust.
    this.#yChildren.delete(0, this.#yChildren.length);
    this.#yChildren.push(content);
  }

  indexOf(content: string): number {
    for (let i = 0; i < this.length; i++) {
      if (this.get(i) === content) {
        return i;
      }
    }
    return -1;
  }

  includes(content: string): boolean {
    return this.indexOf(content) !== -1;
  }

  forEach(f: (arg0: string, arg1: number, arg2: YChildren) => void) {
    this.#yChildren.forEach(f);
  }

  toArray(): string[] {
    return this.#yChildren.toArray();
  }

  toJSON(): string[] {
    return this.#yChildren.toJSON();
  }
}
