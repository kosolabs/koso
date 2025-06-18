import { CircleSlash, type Icon } from "@lucide/svelte";
import { untrack } from "svelte";
import type { Shortcut } from "../shortcut";

type ActionProps = {
  id: string;
  callback: () => void;
  category?: string;
  name?: string;
  description?: string;
  icon?: typeof Icon;
  enabled?: () => boolean;
  selected?: () => boolean;
  shortcut?: Shortcut;
};

export class Action {
  id: string;
  callback: () => void;
  category: string;
  name: string;
  description: string;
  icon: typeof Icon;
  enabled: () => boolean;
  selected?: () => boolean;
  shortcut?: Shortcut;

  constructor({
    id,
    callback,
    category,
    name,
    description,
    icon = CircleSlash,
    enabled = () => true,
    selected,
    shortcut,
  }: ActionProps) {
    this.id = id;
    this.callback = callback;
    this.category = category ?? "";
    this.name = name ?? id;
    this.description = description ?? name ?? id;
    this.icon = icon;
    this.enabled = enabled;
    this.selected = selected;
    this.shortcut = shortcut;
  }
}

export class Registry {
  #actions: Action[] = $state([]);
  #shortcuts: Record<string, Action> = {};

  get actions(): Action[] {
    return this.#actions;
  }

  get(id: string): Action | undefined {
    for (const action of this.#actions) {
      if (action.id === id) {
        return action;
      }
    }
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
        if (this.#actions.find((existing) => existing.id === action.id)) {
          throw new Error(`${action.id} is already registered`);
        }
      });
      if (action.shortcut) {
        if (action.shortcut.toString() in this.#shortcuts) {
          throw new Error(`${action.shortcut} is already registered`);
        }
        this.#shortcuts[action.shortcut.toString()] = action;
      }
    }
    this.#actions.unshift(...actions);
    return () => this.unregister(...actions);
  }

  unregister(...actions: Action[]) {
    for (const action of actions) {
      const index = this.#actions.indexOf(action);
      if (index !== -1) {
        this.#actions.splice(index, 1);
        if (action.shortcut) {
          delete this.#shortcuts[action.shortcut.toString()];
        }
      }
    }
  }
}
