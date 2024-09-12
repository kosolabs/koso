import { writable, type Updater, type Writable } from "svelte/store";

export function storable<T>(
  key: string,
  init: T,
  parse: (data: string) => T = (data) => (data ? JSON.parse(data) : init),
  serialize: (value: T) => string = (value) => JSON.stringify(value),
): Writable<T> {
  const data = localStorage.getItem(key);
  const store = writable<T>(data ? parse(data) : init);

  function save(value: T): T {
    localStorage.setItem(key, serialize(value));
    return value;
  }

  function set(value: T) {
    store.set(save(value));
  }

  function update(updater: Updater<T>) {
    store.update((prev) => save(updater(prev)));
  }

  return {
    subscribe: store.subscribe,
    set,
    update,
  };
}
