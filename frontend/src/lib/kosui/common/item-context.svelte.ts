import { Bindable } from "../bindable.svelte";
import { OrderedHTMLElements } from "../ordered-html-elements";
import { Shortcut } from "../shortcut";

export class ItemContext {
  #input = new Bindable<string>("");
  #items: OrderedHTMLElements = new OrderedHTMLElements();
  #focused: HTMLElement | undefined = $state();

  get input(): string {
    return this.#input.value;
  }

  set input(value: string) {
    this.#input.value = value;
  }

  bindInput(getInput: () => string, setInput: (val: string) => void) {
    this.#input.bind(getInput, setInput);
  }

  get items(): HTMLElement[] {
    return this.#items.items;
  }

  add(item: HTMLElement) {
    this.#items.add(item);
    this.focused = this.#input.value === "" ? undefined : this.items[0];
    return () => this.delete(item);
  }

  delete(item: HTMLElement) {
    this.#items.delete(item);
    this.focused = this.#input.value === "" ? undefined : this.items[0];
  }

  get focused(): HTMLElement | undefined {
    return this.#focused;
  }

  set focused(value: HTMLElement | undefined) {
    this.#focused = value;
  }

  handleKeyDown(event: KeyboardEvent) {
    if (!this.items) return;
    if (Shortcut.ARROW_UP.matches(event)) {
      if (this.items.length !== 0) {
        if (!this.focused) {
          this.focused = this.items[this.items.length - 1];
          this.focused.scrollIntoView(false);
        } else {
          const index = this.items.indexOf(this.focused);
          this.focused =
            this.items[(index - 1 + this.items.length) % this.items.length];
          this.focused.scrollIntoView(false);
        }
      }
      event.preventDefault();
      event.stopImmediatePropagation();
    } else if (Shortcut.ARROW_DOWN.matches(event)) {
      if (this.items.length !== 0) {
        if (!this.focused) {
          this.focused = this.items[0];
          this.focused.scrollIntoView(false);
        } else {
          const index = this.items.indexOf(this.focused);
          this.focused = this.items[(index + 1) % this.items.length];
          this.focused.scrollIntoView(false);
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
