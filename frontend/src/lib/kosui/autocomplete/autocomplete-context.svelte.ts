import { getContext, setContext } from "svelte";
import { Bindable } from "../bindable.svelte";
import { OrderedHTMLElements } from "../ordered-html-elements";
import { Shortcut } from "../shortcut";

export class AutocompleteContext {
  #open = new Bindable<boolean>(false);
  #input = new Bindable<string>("");
  #anchorEl = new Bindable<HTMLElement | undefined>(undefined);
  #items = new OrderedHTMLElements();
  focused: HTMLElement | undefined = $state();

  get open(): boolean {
    return this.#open.value;
  }

  set open(value: boolean) {
    this.#open.value = value;
  }

  bindOpen(getOpen: () => boolean, setOpen: (val: boolean) => void) {
    this.#open.bind(getOpen, setOpen);
  }

  get input(): string {
    return this.#input.value;
  }

  set input(value: string) {
    this.#input.value = value;
  }

  bindInput(getInput: () => string, setInput: (val: string) => void) {
    this.#input.bind(getInput, setInput);
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

  get items(): HTMLElement[] {
    return this.#items.items;
  }

  add(item: HTMLElement) {
    this.#items.add(item);
    this.focused = this.input === "" ? undefined : this.items[0];
    return () => this.delete(item);
  }

  delete(item: HTMLElement) {
    this.#items.delete(item);
    this.focused = this.input === "" ? undefined : this.items[0];
  }

  handleKeyDown(event: KeyboardEvent) {
    if (!this.items) return;
    if (Shortcut.ARROW_UP.matches(event)) {
      if (this.items.length !== 0) {
        if (!this.focused) {
          this.focused = this.items[this.items.length - 1];
        } else {
          const index = this.items.indexOf(this.focused);
          this.focused =
            this.items[(index - 1 + this.items.length) % this.items.length];
        }
      }
      event.preventDefault();
      event.stopImmediatePropagation();
    } else if (Shortcut.ARROW_DOWN.matches(event)) {
      if (this.items.length !== 0) {
        if (!this.focused) {
          this.focused = this.items[0];
        } else {
          const index = this.items.indexOf(this.focused);
          this.focused = this.items[(index + 1) % this.items.length];
        }
      }
      event.preventDefault();
      event.stopImmediatePropagation();
    } else if (Shortcut.ENTER.matches(event)) {
      if (this.focused) {
        this.focused.click();
        event.preventDefault();
        event.stopImmediatePropagation();
      }
    }
  }
}

export function newAutocompleteContext() {
  return setAutocompleteContext(new AutocompleteContext());
}

export function setAutocompleteContext(
  state: AutocompleteContext,
): AutocompleteContext {
  return setContext<AutocompleteContext>(AutocompleteContext, state);
}

export function getAutocompleteContext(): AutocompleteContext {
  return getContext<AutocompleteContext>(AutocompleteContext);
}
