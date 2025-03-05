import type { EventHandler } from "svelte/elements";
import { on } from "svelte/events";
import { runEventHandlers } from "./merge-props";
import { noop } from "./utils";

type EventType = keyof DocumentEventMap;

type Listener<K extends EventType> = EventHandler<DocumentEventMap[K]>;

function entries<T extends object>(obj: T): [keyof T, T[keyof T]][] {
  return Object.entries(obj) as [keyof T, T[keyof T]][];
}

/**
 * Registers event listeners on `window.document` and triggers them in reverse
 * order.
 */
export class EventManager {
  #listeners: { [K in EventType]?: Listener<K>[] } = {};
  #destroyers: { [K in EventType]?: () => void } = {};

  on<K extends EventType>(type: K, listener?: Listener<K>) {
    if (listener === undefined) return noop;
    this.#listeners[type] ??= [];
    if (this.#listeners[type].length === 0) {
      this.#destroyers[type] = on(document, type, (event) => {
        runEventHandlers(
          document,
          event as Event & { currentTarget: EventTarget & Element },
          ...(this.#listeners[type] as EventHandler[]),
        );
      });
    }
    this.#listeners[type].unshift(listener);
    return () => this.remove(type, listener);
  }

  remove<K extends EventType>(type: K, listener?: Listener<K>) {
    if (listener === undefined || this.#listeners[type] === undefined) {
      return;
    }
    const index = this.#listeners[type].indexOf(listener);
    if (index > -1) {
      this.#listeners[type].splice(index, 1);
    }
    if (this.#listeners[type].length === 0) {
      this.#destroyers[type]?.();
      delete this.#destroyers[type];
      delete this.#listeners[type];
    }
  }

  destroy() {
    this.#listeners = {};
    for (const [type, destroy] of entries(this.#destroyers)) {
      destroy?.();
      delete this.#destroyers[type];
    }
  }
}
