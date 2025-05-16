import { getContext, setContext } from "svelte";
import { Bindable } from "../bindable.svelte";
import { ItemContext } from "../common";

export class AutocompleteContext extends ItemContext {
  #open = new Bindable<boolean>(false);
  #anchorEl = new Bindable<HTMLElement | undefined>(undefined);

  get open(): boolean {
    return this.#open.value;
  }

  set open(value: boolean) {
    this.#open.value = value;
  }

  bindOpen(getOpen: () => boolean, setOpen: (val: boolean) => void) {
    this.#open.bind(getOpen, setOpen);
  }

  get anchorEl(): HTMLElement | undefined {
    return this.#anchorEl.value;
  }

  set anchorEl(value: HTMLElement | undefined) {
    this.#anchorEl.value = value;
  }

  bindAnchorEl(
    getAnchorEl: () => HTMLElement | undefined,
    setAnchorEl: (val: HTMLElement | undefined) => void,
  ) {
    this.#anchorEl.bind(getAnchorEl, setAnchorEl);
  }
}

export function setAutocompleteContext(
  state: AutocompleteContext,
): AutocompleteContext {
  return setContext<AutocompleteContext>(AutocompleteContext, state);
}

export function getAutocompleteContext(): AutocompleteContext {
  const ctx = getContext<AutocompleteContext>(AutocompleteContext);
  if (!ctx) throw new Error("AutocompleteContext is undefined");
  return ctx;
}
