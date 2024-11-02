import * as Y from "yjs";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type YEvent = Y.YEvent<any>;
export type YTaskProps = Y.Array<string> | string | number | null;

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
};
export type Status = "Not Started" | "In Progress" | "Done";

export class YGraph {
  #yGraph: Y.Map<Y.Map<YTaskProps>>;

  constructor(yGraph: Y.Map<Y.Map<YTaskProps>>) {
    this.#yGraph = yGraph;
  }

  get size() {
    return this.#yGraph.size;
  }

  taskIds(): IterableIterator<string> {
    return this.#yGraph.keys();
  }

  *tasks(): IterableIterator<YTask> {
    for (const taskId of this.taskIds()) {
      yield this.get(taskId);
    }
  }

  delete(taskId: string) {
    this.#yGraph.delete(taskId);
  }

  set(task: Task): YTask {
    const newTask = new Y.Map<YTaskProps>([
      ["id", task.id],
      ["num", task.num],
      ["name", task.name],
      ["children", Y.Array.from(task.children)],
      ["reporter", task.reporter],
      ["assignee", task.assignee],
      ["status", task.status],
      ["statusTime", task.statusTime],
    ]);
    this.#yGraph.set(task.id, newTask);
    return new YTask(newTask);
  }

  has(taskId: string): boolean {
    return this.#yGraph.has(taskId);
  }

  get(taskId: string): YTask {
    const yTask = this.#yGraph.get(taskId);
    if (!yTask) throw new Error(`Unknown Task ID: ${taskId}`);
    return new YTask(yTask);
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

export class YTask {
  #yTask: Y.Map<YTaskProps>;

  constructor(yTask: Y.Map<YTaskProps>) {
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

  get children(): YChildren {
    const yChildren = this.#yTask.get("children") as Y.Array<string>;
    if (!yChildren) throw new Error("yChildren is undefined");
    return new YChildren(yChildren);
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

  get status(): Status | null {
    return (this.#yTask.get("status") as Status) || null;
  }

  set status(value: Status | null) {
    this.#yTask.set("status", value);
  }

  get statusTime(): number | null {
    return (this.#yTask.get("statusTime") as number) || null;
  }

  set statusTime(value: number | null) {
    this.#yTask.set("statusTime", value);
  }

  toJSON(): Task {
    return this.#yTask.toJSON() as Task;
  }
}

export class YChildren {
  #yChildren: Y.Array<string>;

  constructor(yChildren: Y.Array<string>) {
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

  slice(start?: number | undefined, end?: number | undefined): string[] {
    return this.#yChildren.slice(start, end);
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

  indexOf(content: string): number {
    for (let i = 0; i < this.length; i++) {
      if (this.get(i) === content) {
        return i;
      }
    }
    return -1;
  }

  forEach(f: (arg0: string, arg1: number, arg2: Y.Array<string>) => void) {
    this.#yChildren.forEach(f);
  }

  toArray(): string[] {
    return this.#yChildren.toArray();
  }

  toJSON(): string[] {
    return this.#yChildren.toJSON();
  }
}
