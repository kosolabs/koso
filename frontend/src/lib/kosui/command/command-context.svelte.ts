import { getContext, setContext } from "svelte";
import { OrderedHTMLElements } from "../ordered-html-elements.svelte";
import { Shortcut } from "../shortcut";

export class CommandContext {
  el: HTMLElement | undefined = $state();
  #value: string = $state("");
  #setValue: ((val: string) => void) | undefined;
  #items: OrderedHTMLElements = new OrderedHTMLElements();
  focused: HTMLElement | undefined = $state();

  bind(getValue: () => string, setValue: (val: string) => void) {
    this.#value = getValue();
    this.#setValue = setValue;

    $effect(() => {
      if (this.#value !== getValue()) {
        this.#value = getValue();
      }
    });
  }

  get value(): string {
    return this.#value;
  }

  set value(value: string) {
    if (this.#value !== value) {
      this.#value = value;
      this.#setValue?.(value);
    }
  }

  get items(): HTMLElement[] {
    return this.#items.items;
  }

  add(item: HTMLElement) {
    this.#items.add(item);
    this.focused = this.#value === "" ? undefined : this.items[0];
    return () => this.delete(item);
  }

  delete(item: HTMLElement) {
    this.#items.delete(item);
    this.focused = this.#value === "" ? undefined : this.items[0];
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

export function newCommandContext() {
  return setCommandContext(new CommandContext());
}

export function setCommandContext(state: CommandContext): CommandContext {
  return setContext<CommandContext>(CommandContext, state);
}

export function getCommandContext(): CommandContext {
  return getContext<CommandContext>(CommandContext);
}
