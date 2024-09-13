type Modifiers = {
  altKey?: boolean;
  ctrlKey?: boolean;
  metaKey?: boolean;
  shiftKey?: boolean;
};

export class KeyBinding {
  key: string;
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;

  constructor(
    key: string,
    {
      altKey = false,
      ctrlKey = false,
      metaKey = false,
      shiftKey = false,
    }: Modifiers = {},
  ) {
    this.key = key;
    this.altKey = altKey;
    this.ctrlKey = ctrlKey;
    this.metaKey = metaKey;
    this.shiftKey = shiftKey;
  }

  equals(event: KeyboardEvent): boolean {
    return (
      this.key === event.key &&
      this.altKey === event.altKey &&
      this.ctrlKey === event.ctrlKey &&
      this.metaKey === event.metaKey &&
      this.shiftKey === event.shiftKey
    );
  }

  static INDENT_NODE = new KeyBinding("ArrowRight", { altKey: true });
  static UNDENT_NODE = new KeyBinding("ArrowLeft", { altKey: true });
  static MOVE_NODE_UP = new KeyBinding("ArrowUp", { altKey: true });
  static MOVE_NODE_DOWN = new KeyBinding("ArrowDown", { altKey: true });
  static EXPAND_NODE = new KeyBinding("ArrowRight");
  static COLLAPSE_NODE = new KeyBinding("ArrowLeft");
  static SELECT_PREV_NODE = new KeyBinding("ArrowUp");
  static SELECT_NEXT_NODE = new KeyBinding("ArrowDown");
  static UNDO = new KeyBinding("z", { metaKey: true });
  static REDO = new KeyBinding("z", {
    metaKey: true,
    shiftKey: true,
  });
}
