export class Bindable<T> {
  #value: T = $state()!;
  #setValue: ((value: T) => void) | undefined;

  constructor(value: T) {
    this.#value = value;
  }

  bind(getValue: () => T, setValue: (value: T) => void) {
    this.#value = getValue();
    this.#setValue = setValue;

    $effect(() => {
      if (this.#value !== getValue()) {
        this.#value = getValue();
      }
    });
  }

  get value(): T {
    return this.#value;
  }

  set value(value: T) {
    if (this.#value !== value) {
      this.#value = value;
      this.#setValue?.(value);
    }
  }
}
