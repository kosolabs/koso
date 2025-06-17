import type { User } from "$lib/users";
import { YTaskProxy, type Iteration } from "$lib/yproxy";
import { Record } from "immutable";
import { getContext, setContext } from "svelte";
import * as Y from "yjs";
import type { Koso, Progress } from "./koso.svelte";

export type Reason =
  | {
      name: "Actionable";
      actions: {
        done: number;
        block: number;
        unassign: number;
      };
    }
  | {
      name: "Ready";
      actions: {
        assign: number;
      };
    }
  | {
      name: "ParentOwner";
      actions: {
        assign: number;
        ready: number;
      };
      parents: YTaskProxy[];
    }
  | {
      name: "NeedsEstimate";
      actions: {
        estimate: number;
        assign: number;
      };
      iteration: Iteration;
    };

export type ActionItem = {
  task: YTaskProxy;
  progress: Progress;
  reasons: Reason[];
};

function reasonOrder(reason: Reason): number {
  switch (reason.name) {
    case "Actionable":
      return 1;
    case "Ready":
      return 2;
    default:
      return 0;
  }
}

export class InboxContext {
  #me: User;
  #koso: Koso;
  #yUndoManager: Y.UndoManager;

  #tasks: ActionItem[] = $derived(this.#getActionItems());
  #triage: ActionItem[] = $derived(
    this.#tasks.filter(
      (item) =>
        item.reasons[0].name !== "Actionable" &&
        item.reasons[0].name !== "Ready",
    ),
  );
  #actionable: ActionItem[] = $derived(
    this.#tasks.filter((item) => item.reasons[0].name === "Actionable"),
  );
  #ready: ActionItem[] = $derived(
    this.#tasks.filter((item) => item.reasons[0].name === "Ready"),
  );

  #selectedRaw: Selected = $state(Selected.default());
  #selected: YTaskProxy | undefined = $derived.by(() => {
    const taskId = this.#selectedRaw.taskId;
    return taskId ? this.#koso.getTask(taskId) : undefined;
  });

  constructor(me: User, koso: Koso) {
    this.#me = me;
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
    if (this.#triage.length > 0 || this.#actionable.length > 0) {
      return [...this.#triage, ...this.#actionable];
    } else {
      return this.#ready;
    }
  }

  hasTriage(): boolean {
    return this.#triage.length > 0;
  }

  hasActionable(): boolean {
    return this.#actionable.length > 0;
  }

  hasReady(): boolean {
    return this.#ready.length > 0;
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
      const progress = this.#koso.getProgress(task.id);
      const reasons = this.#getActionableReasons(task, {
        progress,
        iterations: this.#koso.getCurrentIterations(),
      });
      if (reasons.length) {
        items.push({ task, progress, reasons });
      }
    }

    return items
      .map((item) => ({
        item,
        reason: reasonOrder(item.reasons[0]),
        status: this.#koso.getStatusOrder(item.progress.status),
        score: item.reasons
          .map((reason) =>
            Object.values(reason.actions).reduce((a, b) => Math.max(a, b)),
          )
          .reduce((a, b) => a + b),
        num: Number(item.task.num) || 0,
      }))
      .sort(
        (a, b) =>
          // Triage items first, followed by action and ready items
          a.reason - b.reason ||
          // Status next
          a.status - b.status ||
          // Scores, descending
          b.score - a.score ||
          // Stable sort by task number, ascending
          a.num - b.num,
      )
      .map(({ item }) => item);
  }

  #calculateIterationScore(iteration: Iteration): number {
    const deadline = iteration.deadline;
    const daysLeft = (deadline - Date.now()) / (1000 * 60 * 60 * 24);
    const { max, floor, exp, abs } = Math;
    return max(0, floor(20 * exp(-0.1 * abs(daysLeft - 14))));
  }

  #getActionableReasons(
    task: YTaskProxy,
    context: {
      progress: Progress;
      iterations: Iteration[];
    },
  ): Reason[] {
    const reasons: Reason[] = [];

    if (task.id === "root" || task.archived) {
      return reasons;
    }

    // A task is part of the current iteration and doesn't have an estimate
    for (const iteration of context.iterations) {
      //  If the task is not part of the iteration, skip it
      if (!this.#koso.hasDescendant(iteration.id, task.id)) {
        continue;
      }

      if (
        (task.assignee === null || task.assignee === this.#me.email) &&
        task.isTask() &&
        !context.progress.isComplete() &&
        task.estimate === null
      ) {
        reasons.push({
          name: "NeedsEstimate",
          actions: {
            estimate: this.#calculateIterationScore(iteration),
            assign: 1,
          },
          iteration,
        });
      }

      // A task is in an iteration, is ready, and is not assigned to anyone
      if (
        task.assignee === null &&
        (task.isTask() || task.isManaged()) &&
        context.progress.isReady()
      ) {
        reasons.push({
          name: "Ready",
          actions: {
            assign: 1,
          },
        });
      }
    }

    // A task is unassigned and one of its rollup parents is assigned to the user
    if (
      task.assignee === null &&
      !task.isManaged() &&
      !context.progress.isReady() &&
      !context.progress.isComplete() &&
      !context.progress.isBlocked()
    ) {
      const parents = this.#koso
        .getParents(task.id)
        .filter((parent) => parent.isRollup())
        .filter((parent) => parent.assignee === this.#me.email);
      if (parents.length) {
        reasons.push({
          name: "ParentOwner",
          actions: {
            ready: 3,
            assign: 1,
          },
          parents,
        });
      }
    }

    // A task is unblocked, incomplete and assigned to the user
    if (
      task.assignee === this.#me.email &&
      (task.isTask() || task.isManaged()) &&
      !context.progress.isComplete() &&
      !context.progress.isBlocked()
    ) {
      reasons.push({
        name: "Actionable",
        actions: {
          done: task.estimate ?? 1,
          block: Math.min(3, task.estimate ?? 1),
          unassign: 1,
        },
      });
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
