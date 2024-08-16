import type { Unsubscriber } from "svelte/motion";
import {
  writable,
  type Readable,
  type Subscriber,
  type Writable,
} from "svelte/store";

export function storedWritable<T>(
  prefix: string,
  scope: Readable<string>,
  init: T,
  parse: (data: string) => T = (data) => (data ? JSON.parse(data) : init),
  serialize: (value: T) => string = (value) => JSON.stringify(value),
): Writable<T> {
  const result = writable<T>(init);
  const subscribers = new Set();

  function subscribe(run: Subscriber<T>) {
    subscribers.add(run);
    const unsubscribers: Unsubscriber[] = [];

    if (subscribers.size === 1) {
      let key: string | null;
      unsubscribers.push(
        scope.subscribe((scope) => {
          if (!scope) return;
          key = `${prefix}${scope}`;
          const data = localStorage.getItem(key);
          const value = data ? parse(data) : init;
          result.set(value);
        }),
      );
      unsubscribers.push(
        result.subscribe((value) => {
          if (key !== null) {
            localStorage.setItem(key, serialize(value));
          }
        }),
      );
    }
    unsubscribers.push(result.subscribe(run));

    return () => {
      subscribers.delete(run);
      if (subscribers.size === 0) {
        unsubscribers.forEach((unsubscribe) => unsubscribe());
      }
    };
  }

  return {
    set: result.set,
    update: result.update,
    subscribe: subscribe,
  };
}
