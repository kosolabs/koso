const keys: { [key: string]: string } = {
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  ArrowUp: "↑",
  Alt: "⌥",
  Backspace: "⌫",
  CapsLock: "⇪",
  Control: "⌃",
  Dead: "☠️",
  Delete: "⌦",
  End: "⇲",
  Enter: "⏎",
  Escape: "⎋",
  Home: "⌂",
  Meta: "⌘",
  PageDown: "⇟",
  PageUp: "⇞",
  Shift: "⇧",
  Tab: "↹",
  " ": "␣",
};

for (let i = 1; i <= 12; i++) {
  keys[`F${i}`] = `F${i}`;
}

export class Shortcut {
  key: string;
  alt: boolean;
  ctrl: boolean;
  meta: boolean;
  shift: boolean;

  constructor(shortcut: {
    key: string;
    alt?: boolean;
    ctrl?: boolean;
    meta?: boolean;
    shift?: boolean;
  }) {
    this.key = shortcut.key;
    this.alt = shortcut.alt ?? false;
    this.ctrl = shortcut.ctrl ?? false;
    this.meta = shortcut.meta ?? false;
    this.shift = shortcut.shift ?? false;
  }

  matches(event: KeyboardEvent): boolean {
    return (
      this.key === event.key &&
      this.alt === event.altKey &&
      this.ctrl === event.ctrlKey &&
      this.meta === event.metaKey &&
      this.shift === event.shiftKey
    );
  }

  toChar(): string {
    if (this.key !== " " && this.key.length === 1) {
      return this.key.toUpperCase();
    }
    const result = keys[this.key];
    if (!result) {
      console.warn(`${this.key} was not be mapped to a character`);
      return this.key;
    }
    return result;
  }

  toString(): string {
    return [...this].join("");
  }

  *[Symbol.iterator](): Iterator<string> {
    if (this.ctrl) yield "⌃";
    if (this.shift) yield "⇧";
    if (this.alt) yield "⌥";
    if (this.meta) yield "⌘";
    yield this.toChar();
  }

  static isChar(event: KeyboardEvent): boolean {
    return (
      event.key.length === 1 &&
      !event.ctrlKey &&
      !event.altKey &&
      !event.metaKey
    );
  }

  static fromEvent(event: KeyboardEvent): Shortcut {
    return new Shortcut({
      key: event.key,
      alt: event.altKey,
      ctrl: event.ctrlKey,
      meta: event.metaKey,
      shift: event.shiftKey,
    });
  }

  static ARROW_UP = new Shortcut({ key: "ArrowUp" });
  static ARROW_DOWN = new Shortcut({ key: "ArrowDown" });
  static END = new Shortcut({ key: "End" });
  static ENTER = new Shortcut({ key: "Enter" });
  static ESCAPE = new Shortcut({ key: "Escape" });
  static HOME = new Shortcut({ key: "Home" });
  static SPACE = new Shortcut({ key: " " });
  static TAB_BACKWARD = new Shortcut({ key: "Tab", shift: true });
  static TAB_FORWARD = new Shortcut({ key: "Tab" });
}
