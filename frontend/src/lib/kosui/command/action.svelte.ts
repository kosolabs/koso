import { CircleSlash, type Icon } from "lucide-svelte";
import { untrack } from "svelte";
import type { Shortcut } from "../shortcut";

type ActionProps<T extends string> = {
  id: T;
  callback: () => void;
  title?: string;
  description?: string;
  icon?: typeof Icon;
  enabled?: () => boolean;
  shortcut?: Shortcut;
};

export class Action<T extends string> {
  id: T;
  callback: () => void;
  title: string;
  description: string;
  icon: typeof Icon;
  enabled: () => boolean;
  shortcut?: Shortcut;

  constructor({
    id,
    callback,
    title,
    description,
    icon = CircleSlash,
    enabled = () => true,
    shortcut,
  }: ActionProps<T>) {
    this.id = id;
    this.callback = callback;
    this.title = title || id;
    this.description = description || title || id;
    this.icon = icon;
    this.enabled = enabled;
    this.shortcut = shortcut;
  }
}

export class Registry<T extends string> {
  #actions: Record<string, Action<T>> = $state({});
  #shortcuts: Record<string, Action<T>> = {};

  get actions(): Action<T>[] {
    return Object.values(this.#actions);
  }

  get(id: T): Action<T> | undefined {
    return this.#actions[id];
  }

  getByShortcut(shortcut: Shortcut): Action<T> | undefined {
    return this.#shortcuts[shortcut.toString()];
  }

  call(id: T) {
    const action = this.get(id);
    if (!action) {
      throw new Error(`No action named "${id}" is registered`);
    }
    action.callback();
  }

  register(...actions: Action<T>[]) {
    for (const action of actions) {
      untrack(() => {
        if (action.id in this.#actions) {
          throw new Error(`${action.id} is already registered`);
        }
      });
      this.#actions[action.id] = action;
      if (action.shortcut) {
        if (action.shortcut.toString() in this.#shortcuts) {
          throw new Error(`${action.shortcut} is already registered`);
        }
        this.#shortcuts[action.shortcut.toString()] = action;
      }
    }
    return () => this.unregister(...actions);
  }

  unregister(...actions: Action<T>[]) {
    for (const action of actions) {
      delete this.#actions[action.id];
      if (action.shortcut) {
        delete this.#shortcuts[action.shortcut.toString()];
      }
    }
  }
}
