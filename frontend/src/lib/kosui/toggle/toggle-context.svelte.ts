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

export function setToggleContext<T>(ctx: ToggleContext<T>): ToggleContext<T> {
  return setContext<ToggleContext<T>>(ToggleContext, ctx);
}

export function getToggleContext<T>(): ToggleContext<T> {
  const ctx = getContext<ToggleContext<T>>(ToggleContext);
  if (!ctx) throw new Error("ToggleContext is undefined");
  return ctx;
}
