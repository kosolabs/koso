import { Map, Record } from "immutable";

type ShortcutProps = {
  key: string;
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
};
const ShortcutRecord = Record<ShortcutProps>({
  key: "",
  altKey: false,
  ctrlKey: false,
  metaKey: false,
  shiftKey: false,
});

export class Shortcut extends ShortcutRecord {
  matches(event: KeyboardEvent): boolean {
    return (
      this.key === event.key &&
      this.altKey === event.altKey &&
      this.ctrlKey === event.ctrlKey &&
      this.metaKey === event.metaKey &&
      this.shiftKey === event.shiftKey
    );
  }

  toString(): string {
    return (
      (this.ctrlKey ? "⌃" : "") +
      (this.shiftKey ? "⇧" : "") +
      (this.altKey ? "⌥" : "") +
      (this.metaKey ? "⌘" : "") +
      (this.key.length === 1 ? this.key.toUpperCase() : this.key)
    );
  }

  static fromEvent(event: KeyboardEvent): Shortcut {
    return new Shortcut({
      key: event.key,
      altKey: event.altKey,
      ctrlKey: event.ctrlKey,
      metaKey: event.metaKey,
      shiftKey: event.shiftKey,
    });
  }

  static SHOW_COMMAND_PALETTE = new Shortcut({
    key: "p",
    shiftKey: true,
    metaKey: true,
  });
  static SAVE_EDITABLE = new Shortcut({ key: "Enter" });
  static REVERT_EDITABLE = new Shortcut({ key: "Escape" });
  static EDIT_NODE = new Shortcut({ key: "Enter" });
  static CANCEL_SELECTION = new Shortcut({ key: "Escape" });
  static INSERT_NODE = new Shortcut({ key: "Enter", shiftKey: true });
  static REMOVE_NODE = new Shortcut({ key: "Delete" });
  static INSERT_CHILD_NODE = new Shortcut({
    key: "Enter",
    altKey: true,
    shiftKey: true,
  });
  static MOVE_NODE_UP = new Shortcut({ key: "ArrowUp", altKey: true });
  static MOVE_NODE_DOWN = new Shortcut({ key: "ArrowDown", altKey: true });
  static MOVE_NODE_ROW_UP = new Shortcut({
    key: "ArrowUp",
    altKey: true,
    shiftKey: true,
  });
  static MOVE_NODE_ROW_DOWN = new Shortcut({
    key: "ArrowDown",
    altKey: true,
    shiftKey: true,
  });
  static INDENT_NODE = new Shortcut({ key: "ArrowRight", altKey: true });
  static UNDENT_NODE = new Shortcut({ key: "ArrowLeft", altKey: true });
  static INDENT_NODE_SHIFT = new Shortcut({
    key: "ArrowRight",
    altKey: true,
    shiftKey: true,
  });
  static UNDENT_NODE_SHIFT = new Shortcut({
    key: "ArrowLeft",
    altKey: true,
    shiftKey: true,
  });
  static EXPAND_NODE = new Shortcut({ key: "ArrowRight" });
  static COLLAPSE_NODE = new Shortcut({ key: "ArrowLeft" });
  static SELECT_PREV_NODE = new Shortcut({ key: "ArrowUp" });
  static SELECT_NEXT_NODE = new Shortcut({ key: "ArrowDown" });
  static UNDO = new Shortcut({ key: "z", metaKey: true });
  static REDO = new Shortcut({ key: "z", metaKey: true, shiftKey: true });
}

export class ShortcutRegistry {
  registry: Map<Shortcut, Action>;

  constructor(actions: Action[]) {
    this.registry = Map<Shortcut, Action>(
      actions
        .filter((action) => action.shortcut)
        .map((action) => [action.shortcut!, action]),
    );
  }

  handle(event: KeyboardEvent): boolean {
    const action = this.registry.get(Shortcut.fromEvent(event));
    if (!action || !action.enabled()) return false;
    action.callback();
    event.preventDefault();
    return true;
  }
}

export type Action = {
  title: string;
  // TODO: Use Component once lucide-svelte exports a Svelte 5 compatible type
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  icon: any;
  callback: () => void;
  toolbar?: boolean;
  enabled: () => boolean;
  shortcut?: Shortcut;
};