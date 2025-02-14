export class Box<T> {
  #value: T | undefined = $state();

  get value(): T | undefined {
    return this.#value;
  }

  set value(value: T | undefined) {
    this.#value = value;
  }

  apply(value: T | undefined) {
    this.#value = value;
  }
}
