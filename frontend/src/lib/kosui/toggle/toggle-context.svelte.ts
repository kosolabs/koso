import { getContext, setContext } from "svelte";

export class ToggleContext<T> {
  #value: T | undefined = $state(undefined);
  setValue: (val: T | undefined) => void;

  constructor(
    getValue: () => T | undefined,
    setValue: (val: T | undefined) => void,
  ) {
    this.#value = getValue();
    this.setValue = setValue;

    $effect(() => {
      if (this.#value !== getValue()) {
        this.#value = getValue();
      }
    });
  }

  get value(): T | undefined {
    return this.#value;
  }

  set value(value: T | undefined) {
    if (this.#value !== value) {
      this.#value = value;
      this.setValue(value);
    }
  }
}

export function newToggleContext<T>(
  getValue: () => T | undefined,
  setValue: (val: T | undefined) => void,
) {
  return setToggleContext(new ToggleContext(getValue, setValue));
}

export function setToggleContext<T>(state: ToggleContext<T>): ToggleContext<T> {
  return setContext<ToggleContext<T>>(ToggleContext, state);
}

export function getToggleContext<T>(): ToggleContext<T> {
  return getContext<ToggleContext<T>>(ToggleContext);
}
