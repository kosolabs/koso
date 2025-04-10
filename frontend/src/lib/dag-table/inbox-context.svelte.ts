import { YTaskProxy } from "$lib/yproxy";
import { Record } from "immutable";
import { getContext, setContext } from "svelte";
import type { DetailPanelStates, Koso } from "./koso.svelte";
import * as Y from "yjs";
import { PanelTopClose, PanelTopOpen, SquarePen } from "lucide-svelte";
import { Action } from "$lib/kosui/command";
import { command, type ActionID } from "$lib/components/ui/command-palette";

export class InboxContext {
  #koso: Koso;
  #yUndoManager: Y.UndoManager;

  #selectedRaw: Selected = $state(Selected.default());
  #selected: YTaskProxy | undefined = $derived.by(() => {
    const task = this.#selectedRaw.task;
    if (!task || this.#koso.tasks.indexOf(task) < 0) {
      return undefined;
    }
    return task;
  });

  #detailPanel: DetailPanelStates = $state("none");

  constructor(koso: Koso) {
    this.#koso = koso;

    this.#yUndoManager = new Y.UndoManager(this.#koso.graph.yGraph, {
      captureTransaction: (txn) => txn.local,
    });
    // Save and restore node selection on undo/redo.
    this.#yUndoManager.on("stack-item-added", (event) => {
      event.stackItem.meta.set("selected-task", this.selectedRaw.task);
    });
    this.#yUndoManager.on("stack-item-popped", (event) => {
      const selected = event.stackItem.meta.get("selected-task");
      if (selected === null || selected.constructor === YTaskProxy) {
        this.selected = selected;
      } else {
        console.warn(
          `Unexpectedly found non-task "selected-task" stack item: ${selected}`,
        );
        this.selected = undefined;
      }
    });

    const actions: Action<ActionID>[] = [
      new Action({
        id: "DetailPanelClose",
        callback: () => (this.detailPanel = "none"),
        title: "Close task description",
        description: "Close / hide the task description markdown panel",
        icon: PanelTopClose,
      }),
      new Action({
        id: "DetailPanelViewer",
        callback: () => (this.detailPanel = "view"),
        title: "View task description",
        description: "Open / show the task description markdown viewer",
        icon: PanelTopOpen,
        enabled: () => !!this.selected,
      }),
      new Action({
        id: "DetailPanelEditor",
        callback: () => (this.detailPanel = "edit"),
        title: "Edit task description",
        description: "Open / show the task description markdown editor",
        icon: SquarePen,
        enabled: () =>
          !!this.selected && this.#koso.isEditable(this.selected.name),
      }),
    ];

    $effect(() => {
      return command.register(...actions);
    });
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

  set selected(task: YTaskProxy | undefined) {
    if (task && task.id === "root") {
      throw new Error("Cannot select root");
    }

    if (task) {
      const index = this.#koso.tasks.indexOf(task);
      if (index === -1) {
        // TODO: This happens when handleRow click is triggered when setting status to done in the inbox.
        // It'd be better if this threw.
        console.warn(`Selected task ${task.id} not found in tasks.`);
        return;
      }
      if (index === 0) {
        throw new Error(`Cannot selected root task ${task.id} at task index 0`);
      }
      this.#selectedRaw = Selected.create(task, index);
    } else {
      this.#selectedRaw = Selected.default();
    }
  }

  select(taskId: string) {
    const task = this.#koso.tasks.find((t) => t.id == taskId);
    if (!task) throw new Error("Expected at least one Node");
    this.selected = task;
  }

  // actions that operate on the UI

  get detailPanel() {
    return this.#detailPanel;
  }

  set detailPanel(value: DetailPanelStates) {
    this.#detailPanel = value;
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

type SelectedProps = { task: YTaskProxy | null; index: number | null };
const SelectedRecord = Record<SelectedProps>({ task: null, index: null });

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

  static create(task: YTaskProxy, index: number) {
    return new Selected({ task, index });
  }
}
const DEFAULT_SELECTED = new Selected({ task: undefined, index: null });
