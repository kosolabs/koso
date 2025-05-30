import { CircleSlash, type Icon } from "lucide-svelte";
import { untrack } from "svelte";
import type { Shortcut } from "../shortcut";

type ActionProps = {
  id: string;
  callback: () => void;
  title?: string;
  description?: string;
  icon?: typeof Icon;
  enabled?: () => boolean;
  shortcut?: Shortcut;
};

export class Action {
  id: string;
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
  }: ActionProps) {
    this.id = id;
    this.callback = callback;
    this.title = title || id;
    this.description = description || title || id;
    this.icon = icon;
    this.enabled = enabled;
    this.shortcut = shortcut;
  }
}

export class Registry {
  #actions: Record<string, Action> = $state({});
  #shortcuts: Record<string, Action> = {};

  get actions(): Action[] {
    return Object.values(this.#actions);
  }

  get(id: string): Action | undefined {
    return this.#actions[id];
  }

  getByShortcut(shortcut: Shortcut): Action | undefined {
    return this.#shortcuts[shortcut.toString()];
  }

  call(id: string) {
    const action = this.get(id);
    if (!action) {
      throw new Error(`No action named "${id}" is registered`);
    }
    action.callback();
  }

  register(...actions: Action[]) {
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

  unregister(...actions: Action[]) {
    for (const action of actions) {
      delete this.#actions[action.id];
      if (action.shortcut) {
        delete this.#shortcuts[action.shortcut.toString()];
      }
    }
  }
}
