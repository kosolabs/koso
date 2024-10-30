import { Map, Record } from "immutable";
import { CircleSlash, Icon } from "lucide-svelte";

type ShortcutProps = {
  key: string;
  alt: boolean;
  ctrl: boolean;
  meta: boolean;
  shift: boolean;
};
const ShortcutRecord = Record<ShortcutProps>({
  key: "",
  alt: false,
  ctrl: false,
  meta: false,
  shift: false,
});

const keys = Map([
  ["ArrowDown", "↓"],
  ["ArrowLeft", "←"],
  ["ArrowRight", "→"],
  ["ArrowUp", "↑"],
  ["Delete", "⌦"],
  ["Enter", "⏎"],
  ["Escape", "⎋"],
  [" ", "␣"],
]);

export class Shortcut extends ShortcutRecord {
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
    const result = keys.get(this.key);
    if (!result) {
      throw new Error(`${this.key} was not be mapped to a character`);
    }
    return result;
  }

  toString(): string {
    return (
      (this.ctrl ? "⌃" : "") +
      (this.shift ? "⇧" : "") +
      (this.alt ? "⌥" : "") +
      (this.meta ? "⌘" : "") +
      this.toChar()
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

  static OK = new Shortcut({ key: "Enter" });
  static CANCEL = new Shortcut({ key: "Escape" });
  static INSERT_NODE = new Shortcut({ key: "Enter", shift: true });
  static INSERT_CHILD_NODE = new Shortcut({
    key: "Enter",
    alt: true,
    shift: true,
  });
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
    if (!action || (action.enabled && !action.enabled())) return false;
    action.callback();
    event.preventDefault();
    return true;
  }
}

type ActionProps = {
  callback: () => void;
  title?: string;
  description?: string;
  icon?: typeof Icon;
  toolbar?: boolean;
  enabled?: () => boolean;
  shortcut?: Shortcut;
};

export class Action {
  callback: () => void;
  title: string;
  description: string;
  icon: typeof Icon;
  toolbar: boolean;
  enabled: () => boolean;
  shortcut?: Shortcut;

  constructor({
    callback,
    title = "Untitled",
    description,
    icon = CircleSlash,
    toolbar = false,
    enabled = () => true,
    shortcut,
  }: ActionProps) {
    this.callback = callback;
    this.title = title;
    this.description = description || title;
    this.icon = icon;
    this.toolbar = toolbar;
    this.enabled = enabled;
    this.shortcut = shortcut;
  }
}
