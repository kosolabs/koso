import { auth } from "$lib/auth.svelte";
import { YTaskProxy } from "$lib/yproxy";
import { Record } from "immutable";
import { getContext, setContext } from "svelte";
import * as Y from "yjs";
import type { Koso } from "./koso.svelte";

export type Reason =
  | {
      name: "Actionable";
    }
  | {
      name: "ParentOwner";
      parents: YTaskProxy[];
    };

export class ActionItem {
  task: YTaskProxy;
  reasons: Reason[];
  // TODO: Add a priority

  constructor(task: YTaskProxy, reasons: Reason[]) {
    this.task = task;
    this.reasons = reasons;
  }
}

export class InboxContext {
  #koso: Koso;
  #yUndoManager: Y.UndoManager;

  #tasks: ActionItem[] = $derived(this.#getActionItems());

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
    const tasks: ActionItem[] = [];
    for (const task of this.#koso.tasks) {
      const reasons = this.#getActionableReasons(task);
      if (reasons.length) {
        tasks.push(new ActionItem(task, reasons));
      }
    }
    return tasks;
  }

  #getActionableReasons(task: YTaskProxy): Reason[] {
    const reasons: Reason[] = [];

    if (task.id === "root") {
      return reasons;
    }

    const progress = this.#koso.getProgress(task.id);

    // A leaf task is unblocked, incomplete and assigned to the user
    if (
      task.assignee === auth.user.email &&
      progress.kind !== "Rollup" &&
      !progress.isComplete() &&
      !progress.isBlocked()
    ) {
      reasons.push({ name: "Actionable" });
    }

    // A task is unassigned and one of its rollup parents is assigned to the user
    if (
      task.assignee === null &&
      !progress.isComplete() &&
      !progress.isBlocked()
    ) {
      const parents = this.#koso
        .getParents(task.id)
        // TODO: Make isRollup() a readable version of the next line
        .filter((parent) => parent.yKind === null)
        .filter((parent) => parent.assignee === auth.user.email);
      if (parents.length) {
        reasons.push({ name: "ParentOwner", parents });
      }
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
