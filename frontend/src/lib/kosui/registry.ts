// export type Item = {};

export class Registry<T> {
  #items: Record<string, T> = {};

  register(id: string, item: T) {
    this.#items[id] = item;
  }

  unregister(id: string) {
    delete this.#items[id];
  }

  get(id: string): T {
    return this.#items[id];
  }

  *values(): IterableIterator<T> {
    for (const key in this.#items) {
      yield this.#items[key];
    }
  }
}
