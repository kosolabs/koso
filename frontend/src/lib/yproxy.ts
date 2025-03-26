import { Set } from "immutable";
import * as Y from "yjs";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type YEvent = Y.YEvent<any>;
export type YGraph = Y.Map<YTask>;
export type YTask = Y.Map<YTaskProps>;
export type YTaskProps = YChildren | string | number | null;
export type YChildren = Y.Array<string>;

export type Graph = { [id: string]: Task };
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
  // The kind of task, either directly associated with Koso
  // or a plugin. e.g. github.
  kind: Kind | null;
  // URL associated with the task. Typically provided by a plugin
  // to associate this task with some external entity.
  // e.g. a Github PR URL.
  url: string | null;
};
export type Status = "Not Started" | "In Progress" | "Done" | "Juggled";
export type Kind = YKind | "Rollup";
export type YKind = "Task" | "github" | "github_pr";
export const unmanagedKinds: Set<Kind> = Set.of("Rollup", "Task");

export type Slice = {
  start?: number;
  end?: number;
  step?: number;
};

export class YGraphProxy {
  #yGraph: YGraph;

  constructor(yGraph: YGraph) {
    this.#yGraph = yGraph;
  }

  get size() {
    return this.#yGraph.size;
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
      ["children", Y.Array.from(task.children)],
      ["reporter", task.reporter],
      ["assignee", task.assignee],
      ["status", task.status],
      ["statusTime", task.statusTime],
      ["kind", task.kind],
      ["url", task.url],
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

export class YTaskProxy {
  #yTask: YTask;

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

  get yKind(): YKind | null {
    return (this.#yTask.get("kind") as YKind) || null;
  }

  set yKind(value: YKind | null) {
    this.#yTask.set("kind", value);
  }

  get url(): string | null {
    return (this.#yTask.get("url") as string) || null;
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
