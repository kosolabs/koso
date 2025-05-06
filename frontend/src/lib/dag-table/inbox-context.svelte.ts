import { auth } from "$lib/auth.svelte";
import { YTaskProxy } from "$lib/yproxy";
import { Record } from "immutable";
import { getContext, setContext } from "svelte";
import * as Y from "yjs";
import type { Koso } from "./koso.svelte";

export class InboxContext {
  #koso: Koso;
  #yUndoManager: Y.UndoManager;

  #tasks: YTaskProxy[] = $derived(this.#visibleTasks());

  #selectedRaw: Selected = $state(Selected.default());
  #selected: YTaskProxy | undefined = $derived.by(() => {
    const taskId = this.#selectedRaw.taskId;
    return taskId ? this.#koso.getTask(taskId) : undefined;
  });

  constructor(koso: Koso) {
    this.#koso = koso;

    this.#yUndoManager = new Y.UndoManager(this.#koso.graph.yGraph, {
      captureTransaction: (txn) => txn.local,
    });
    // Save and restore node selection on undo/redo.
    this.#yUndoManager.on("stack-item-added", (event) => {
      event.stackItem.meta.set("selected-task", this.selectedRaw.taskId);
    });
    this.#yUndoManager.on("stack-item-popped", (event) => {
      const selectedTaskId = event.stackItem.meta.get("selected-task");
      if (selectedTaskId === null || typeof selectedTaskId === "string") {
        this.selected = selectedTaskId;
      } else {
        console.warn(
          `Unexpectedly found non-task "selected-task" stack item: ${selectedTaskId}`,
        );
        this.selected = undefined;
      }
    });
  }

  get koso(): Koso {
    return this.#koso;
  }

  get tasks(): YTaskProxy[] {
    return this.#tasks;
  }

  /**
   * Returns the currently selected task, even if it no longer exists in the
   * tasks list.
   *
   * Most usages should prefer to use the `selected` getter below instead which
   * applies a filter to ensure the task exists.
   */
  get selectedRaw(): Selected {
    return this.#selectedRaw;
  }

  get selected(): YTaskProxy | undefined {
    return this.#selected;
  }

  set selected(taskId: string | undefined) {
    if (taskId && taskId === "root") {
      throw new Error("Cannot select root");
    }

    if (taskId) {
      const index = this.getTaskIndex(taskId);
      if (index === -1) {
        // TODO: This happens when handleRow click is triggered when setting status to done in the inbox.
        // It'd be better if this threw.
        console.warn(`Selected task ${taskId} not found in tasks.`);
        return;
      }
      this.#selectedRaw = Selected.create(taskId, index);
    } else {
      this.#selectedRaw = Selected.default();
    }
  }

  #visibleTasks(): YTaskProxy[] {
    const tasks: YTaskProxy[] = [];
    for (const task of this.#koso.tasks) {
      if (this.#isTaskVisible(task)) {
        tasks.push(task);
      }
    }
    return tasks;
  }

  #isTaskVisible(task: YTaskProxy): boolean {
    if (task.id === "root") {
      return false;
    }

    // Don't show tasks not assigned to the user
    if (task.assignee !== null && task.assignee !== auth.user.email) {
      return false;
    }
    // Don't show rollup tasks where every child is assigned.
    if (
      task.yKind === null &&
      task.children.length > 0 &&
      Array.from(task.children.slice())
        .map((childId) => this.#koso.getTask(childId))
        .every(
          (child) =>
            child.assignee !== null ||
            this.#koso.getProgress(child.id).isComplete(),
        )
    ) {
      return false;
    }

    // Don't show unassigned task where none of the parents are assigned to the user
    if (
      task.assignee === null &&
      this.#koso
        .getParents(task.id)
        .filter((parent) => parent.yKind === null)
        .every((parent) => parent.assignee !== auth.user.email)
    ) {
      return false;
    }
    const progress = this.#koso.getProgress(task.id);
    return !progress.isComplete() && !progress.isBlocked();
  }

  /**
   * Retrieves the index of the task in tasks {@link tasks}, if found, and -1
   * otherwise.
   */
  getTaskIndex(taskId: string): number {
    return this.tasks.findIndex((t) => t.id === taskId);
  }

  undo() {
    this.#yUndoManager.undo();
  }

  redo() {
    this.#yUndoManager.redo();
  }
}

export function newInboxContext(koso: Koso) {
  return setInboxContext(new InboxContext(koso));
}

export function setInboxContext(ctx: InboxContext): InboxContext {
  return setContext<InboxContext>(InboxContext, ctx);
}

export function getInboxContext(): InboxContext {
  return getContext<InboxContext>(InboxContext);
}

type SelectedProps = { taskId: string | null; index: number | null };
const SelectedRecord = Record<SelectedProps>({ taskId: null, index: null });

export class Selected extends SelectedRecord {
  constructor(props: Partial<SelectedProps>) {
    if (!!props.index && props.index < 0) {
      props.index = null;
    }
    super(props);
  }

  static default(): Selected {
    return DEFAULT_SELECTED;
  }

  static create(taskId: string, index: number) {
    return new Selected({ taskId, index });
  }
}
const DEFAULT_SELECTED = new Selected({ taskId: undefined, index: null });
