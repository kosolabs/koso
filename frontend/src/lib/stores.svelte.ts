export function load<T>(
  key: string,
  init: T,
  parse: (data: string) => T = (data) => (data ? JSON.parse(data) : init),
): T {
  const data = localStorage.getItem(key);
  return data ? parse(data) : init;
}

export function save<T>(
  key: string,
  value: T,
  serialize: (value: T) => string = (value) => JSON.stringify(value),
) {
  if (value) {
    localStorage.setItem(key, serialize(value));
  } else {
    localStorage.removeItem(key);
  }
}

export type Storable<T> = {
  value: T;
};

export function useLocalStorage<T>(
  key: string,
  init: T,
  parse: (data: string) => T = (data) => (data ? JSON.parse(data) : init),
  serialize: (value: T) => string = (value) => JSON.stringify(value),
): Storable<T> {
  let value = $state(load(key, init, parse));

  return {
    get value() {
      return value;
    },
    set value(v: T) {
      value = v;
      save(key, value, serialize);
    },
  };
}
