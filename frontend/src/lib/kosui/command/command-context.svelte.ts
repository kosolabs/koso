import { getContext, setContext } from "svelte";
import { Shortcut } from "../shortcut";

export class CommandContext {
  el: HTMLElement | undefined = $state();
  #value: string = $state("");
  #setValue: ((val: string) => void) | undefined;
  #reg: Set<HTMLElement> = new Set();
  items: HTMLElement[] = $state.raw([]);
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

  add(item: HTMLElement) {
    this.#reg.add(item);
    this.#updateItems();
    return () => this.delete(item);
  }

  delete(item: HTMLElement) {
    this.#reg.delete(item);
    this.#updateItems();
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

  #updateItems() {
    const items = this.el
      ? Array.from(this.el.getElementsByTagName("div")).filter(
          (button) => button.role === "option" && this.#reg.has(button),
        )
      : [];
    this.items = items;
    this.focused = this.#value === "" ? undefined : items[0];
    return items;
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
