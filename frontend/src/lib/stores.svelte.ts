type StoreOpts = {
  storage?: Storage;
};

type LoadOpts<T> = {
  decode?: (data: string) => T;
};

type SaveOpts<T> = {
  encode?: (value: T) => string;
};

export function load<T>(
  key: string,
  init: T,
  opts: LoadOpts<T> & StoreOpts,
): T {
  const {
    storage = localStorage,
    decode = (data) => (data ? JSON.parse(data) : init),
  } = opts;
  const data = storage.getItem(key);
  return data ? decode(data) : init;
}

export function save<T>(key: string, value: T, opts: SaveOpts<T> & StoreOpts) {
  const { storage = localStorage, encode = (value) => JSON.stringify(value) } =
    opts;
  if (value) {
    storage.setItem(key, encode(value));
  } else {
    storage.removeItem(key);
  }
}

export type Storable<T> = {
  value: T;
};

export function useLocalStorage<T>(
  key: string,
  init: T,
  opts: LoadOpts<T> & SaveOpts<T> & StoreOpts = {},
): Storable<T> {
  let value = $state(load(key, init, opts));

  return {
    get value() {
      return value;
    },
    set value(v: T) {
      value = v;
      save(key, value, opts);
    },
  };
}

export const loads = (
  key: string,
  init: string | null,
  opts: StoreOpts = {},
): string | null => load(key, init, { decode: (data) => data, ...opts });

export const saves = (
  key: string,
  value: string | null,
  opts: StoreOpts = {},
) => save(key, value, { encode: (value) => String(value), ...opts });
