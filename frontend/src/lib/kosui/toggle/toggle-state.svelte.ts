export class ToggleState {
  #value: string | undefined = $state(undefined);
  #onChange: (val: string | undefined) => void;

  constructor(
    getValue: () => string | undefined,
    onChange: (val: string | undefined) => void,
  ) {
    this.#value = getValue();
    this.#onChange = onChange;

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
      this.#onChange(value);
    }
  }
}
