import { getContext, setContext } from "svelte";

export class ToggleState {
  #value: string | undefined = $state(undefined);
  setValue: (val: string | undefined) => void;

  constructor(
    getValue: () => string | undefined,
    setValue: (val: string | undefined) => void,
  ) {
    this.#value = getValue();
    this.setValue = setValue;

    $effect(() => {
      if (this.#value !== getValue()) {
        this.#value = getValue();
      }
    });
  }

  get value(): string | undefined {
    return this.#value;
  }

  set value(value: string | undefined) {
    if (this.#value !== value) {
      this.#value = value;
      this.setValue(value);
    }
  }
}

export function newToggleContext(
  getValue: () => string | undefined,
  setValue: (val: string | undefined) => void,
) {
  return setToggleContext(new ToggleState(getValue, setValue));
}

export function setToggleContext(state: ToggleState): ToggleState {
  return setContext<ToggleState>(ToggleState, state);
}

export function getToggleContext(): ToggleState {
  return getContext<ToggleState>(ToggleState);
}
