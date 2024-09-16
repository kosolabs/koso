import { Record } from "immutable";

type KeyBindingProps = {
  key: string;
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
};
const KeyBindingRecord = Record<KeyBindingProps>({
  key: "",
  altKey: false,
  ctrlKey: false,
  metaKey: false,
  shiftKey: false,
});

export class KeyBinding extends KeyBindingRecord {
  equals(event: KeyboardEvent): boolean {
    return (
      this.key === event.key &&
      this.altKey === event.altKey &&
      this.ctrlKey === event.ctrlKey &&
      this.metaKey === event.metaKey &&
      this.shiftKey === event.shiftKey
    );
  }

  static fromEvent(event: KeyboardEvent): KeyBinding {
    return new KeyBinding({
      key: event.key,
      altKey: event.altKey,
      ctrlKey: event.ctrlKey,
      metaKey: event.metaKey,
      shiftKey: event.shiftKey,
    });
  }

  static EDIT_NODE = new KeyBinding({ key: "Enter" });
  static CANCEL_SELECTION = new KeyBinding({ key: "Escape" });
  static INSERT_NODE = new KeyBinding({ key: "Enter", shiftKey: true });
  static REMOVE_NODE = new KeyBinding({ key: "Delete" });
  static MOVE_NODE_UP = new KeyBinding({ key: "ArrowUp", altKey: true });
  static MOVE_NODE_DOWN = new KeyBinding({ key: "ArrowDown", altKey: true });
  static INDENT_NODE = new KeyBinding({ key: "ArrowRight", altKey: true });
  static UNDENT_NODE = new KeyBinding({ key: "ArrowLeft", altKey: true });
  static EXPAND_NODE = new KeyBinding({ key: "ArrowRight" });
  static COLLAPSE_NODE = new KeyBinding({ key: "ArrowLeft" });
  static SELECT_PREV_NODE = new KeyBinding({ key: "ArrowUp" });
  static SELECT_NEXT_NODE = new KeyBinding({ key: "ArrowDown" });
  static UNDO = new KeyBinding({ key: "z", metaKey: true });
  static REDO = new KeyBinding({ key: "z", metaKey: true, shiftKey: true });
}
