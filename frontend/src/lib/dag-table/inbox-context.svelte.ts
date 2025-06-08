import type { AuthContext } from "$lib/auth.svelte";
import { YTaskProxy, type Iteration } from "$lib/yproxy";
import { Record } from "immutable";
import { getContext, setContext } from "svelte";
import * as Y from "yjs";
import type { Koso } from "./koso.svelte";

export type Reason = { score: number } & (
  | {
      name: "Actionable";
    }
  | {
      name: "ParentOwner";
      parents: YTaskProxy[];
    }
  | {
      name: "NeedsEstimate";
      iteration: YTaskProxy;
    }
);

export class ActionItem {
  task: YTaskProxy;
  reasons: Reason[];
  score: number;

  constructor(task: YTaskProxy, reasons: Reason[], priority: number) {
    this.task = task;
    this.reasons = reasons;
    this.score = priority;
  }
}

export class InboxContext {
  #auth: AuthContext;
  #koso: Koso;
  #yUndoManager: Y.UndoManager;

  #tasks: ActionItem[] = $derived(this.#getActionItems());

  #selectedRaw: Selected = $state(Selected.default());
  #selected: YTaskProxy | undefined = $derived.by(() => {
    const taskId = this.#selectedRaw.taskId;
    return taskId ? this.#koso.getTask(taskId) : undefined;
  });

  constructor(auth: AuthContext, koso: Koso) {
    this.#auth = auth;
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

  get actionItems(): ActionItem[] {
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

  #getActionItems(): ActionItem[] {
    const items: ActionItem[] = [];
    for (const task of this.#koso.tasks) {
      const reasons = this.#getActionableReasons(task, {
        currentIterations: this.#koso.getCurrentIterations(),
      });
      if (reasons.length) {
        items.push(
          new ActionItem(
            task,
            reasons,
            reasons.map((reason) => reason.score).reduce((a, b) => a + b),
          ),
        );
      }
    }

    return items.sort((a, b) => {
      // Sort first by priority.
      const cmp = b.score - a.score;
      if (cmp !== 0) {
        return cmp;
      }
      // If priorities were equal, sort by number to ensure a stable order.
      return a.task.num.localeCompare(b.task.num);
    });
  }

  #calculateIterationScore(iteration: Iteration): number {
    const deadline = iteration.deadline;
    const daysLeft = (deadline - Date.now()) / (1000 * 60 * 60 * 24);
    const { max, floor, exp, abs } = Math;
    return max(0, floor(20 * exp(-0.1 * abs(daysLeft - 14))));
  }

  #getActionableReasons(
    task: YTaskProxy,
    context: { currentIterations: Iteration[] },
  ): Reason[] {
    const reasons: Reason[] = [];

    if (task.id === "root") {
      return reasons;
    }

    const progress = this.#koso.getProgress(task.id);

    // A task is part of the current iteration and doesn't have an estimate
    for (const iteration of context.currentIterations) {
      if (
        task.assignee === this.#auth.user.email &&
        task.isTask() &&
        !progress.isComplete() &&
        this.#koso.hasDescendant(iteration.id, task.id) &&
        task.estimate === null
      ) {
        reasons.push({
          name: "NeedsEstimate",
          score: this.#calculateIterationScore(iteration),
          iteration,
        });
      }
    }

    // A task is unassigned and one of its rollup parents is assigned to the user
    if (
      task.assignee === null &&
      !task.isManaged() &&
      !progress.isComplete() &&
      !progress.isBlocked()
    ) {
      const parents = this.#koso
        .getParents(task.id)
        .filter((parent) => parent.isRollup())
        .filter((parent) => parent.assignee === this.#auth.user.email);
      if (parents.length) {
        reasons.push({ name: "ParentOwner", score: 10, parents });
      }
    }

    // A leaf task is unblocked, incomplete and assigned to the user
    if (
      task.assignee === this.#auth.user.email &&
      task.isLeaf() &&
      !progress.isComplete() &&
      !progress.isBlocked()
    ) {
      reasons.push({ name: "Actionable", score: task.estimate ?? 1 });
    }

    return reasons;
  }

  /**
   * Retrieves the index of the task in tasks {@link actionItems}, if found, and
   * -1 otherwise.
   */
  getTaskIndex(taskId: string): number {
    return this.actionItems.findIndex((t) => t.task.id === taskId);
  }

  undo() {
    this.#yUndoManager.undo();
  }

  redo() {
    this.#yUndoManager.redo();
  }
}

export function setInboxContext(ctx: InboxContext): InboxContext {
  return setContext<InboxContext>(InboxContext, ctx);
}

export function getInboxContext(): InboxContext {
  const ctx = getContext<InboxContext>(InboxContext);
  if (!ctx) throw new Error("InboxContext is undefined");
  return ctx;
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
