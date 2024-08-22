import { writable, type Writable } from "svelte/store";

export function storable<T>(
  key: string,
  init: T,
  parse: (data: string) => T = (data) => (data ? JSON.parse(data) : init),
  serialize: (value: T) => string = (value) => JSON.stringify(value),
): Writable<T> {
  const data = localStorage.getItem(key);
  const value = data ? parse(data) : init;
  const store = writable<T>(value);

  function set(value: T) {
    localStorage.setItem(key, serialize(value));
    store.set(value);
  }

  return {
    subscribe: store.subscribe,
    set,
    update: (fn) => set(fn(value)),
  };
}
