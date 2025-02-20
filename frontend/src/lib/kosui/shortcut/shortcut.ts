const keys: { [key: string]: string } = {
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  ArrowUp: "↑",
  Alt: "⌥",
  Backspace: "⌫",
  CapsLock: "⇪",
  Control: "⌃",
  Delete: "⌦",
  Enter: "⏎",
  Escape: "⎋",
  Meta: "⌘",
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
    return [...this].join();
  }

  *[Symbol.iterator](): Iterator<string> {
    if (this.ctrl) yield "⌃";
    if (this.shift) yield "⇧";
    if (this.alt) yield "⌥";
    if (this.meta) yield "⌘";
    yield this.toChar();
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
}
