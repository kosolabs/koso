import { getContext, setContext } from "svelte";
import { Bindable } from "../bindable.svelte";
import { OrderedHTMLElements } from "../ordered-html-elements";
import { Shortcut } from "../shortcut";
import { TypingBuffer } from "../utils";

export class MenuContext {
  #open: Bindable<boolean>;
  #anchorEl: Bindable<HTMLElement | undefined>;
  #items = new OrderedHTMLElements();
  focused: HTMLElement | undefined = $state();
  #buffer = new TypingBuffer();

  constructor(
    getOpen: () => boolean,
    setOpen: (val: boolean) => void,
    getAnchorEl: () => HTMLElement | undefined,
    setAnchorEl: (anchorEl: HTMLElement | undefined) => void,
  ) {
    this.#open = new Bindable(getOpen());
    this.#open.bind(getOpen, setOpen);
    this.#anchorEl = new Bindable<HTMLElement | undefined>(getAnchorEl());
    this.#anchorEl.bind(getAnchorEl, setAnchorEl);
  }

  get open(): boolean {
    return this.#open.value;
  }

  set open(value: boolean) {
    this.#open.value = value;
  }

  get anchorEl(): HTMLElement | undefined {
    return this.#anchorEl.value;
  }

  set anchorEl(value: HTMLElement | undefined) {
    this.#anchorEl.value = value;
  }

  close() {
    this.open = false;
    this.anchorEl?.focus();
  }

  get items(): HTMLElement[] {
    return this.#items.items;
  }

  add(item: HTMLElement) {
    this.#items.add(item);
    return () => this.delete(item);
  }

  delete(item: HTMLElement) {
    this.#items.delete(item);
  }

  focus(item?: HTMLElement) {
    if (!item) return;
    this.focused = item;
    this.focused.focus();
  }

  blur() {
    this.focused?.blur();
    this.focused = undefined;
  }

  handleKeyDown(event: KeyboardEvent) {
    if (!this.items) return;
    if (
      Shortcut.ARROW_UP.matches(event) ||
      Shortcut.TAB_BACKWARD.matches(event)
    ) {
      if (!this.focused) {
        this.focus(this.items[this.items.length - 1]);
      } else {
        let activeIndex = this.items.indexOf(this.focused);
        do {
          activeIndex =
            (activeIndex - 1 + this.items.length) % this.items.length;
        } while ((this.items[activeIndex] as HTMLButtonElement).disabled);
        this.focus(this.items[activeIndex]);
      }
      event.preventDefault();
      event.stopImmediatePropagation();
    } else if (
      Shortcut.ARROW_DOWN.matches(event) ||
      Shortcut.TAB_FORWARD.matches(event)
    ) {
      if (!this.focused) {
        this.focus(this.items[0]);
      } else {
        let activeIndex = this.items.indexOf(this.focused);
        do {
          activeIndex = (activeIndex + 1) % this.items.length;
        } while ((this.items[activeIndex] as HTMLButtonElement).disabled);
        this.focus(this.items[activeIndex]);
      }
      event.preventDefault();
      event.stopImmediatePropagation();
    } else if (Shortcut.HOME.matches(event)) {
      this.focus(this.items[0]);
      event.preventDefault();
      event.stopImmediatePropagation();
    } else if (Shortcut.END.matches(event)) {
      this.focus(this.items[this.items.length - 1]);
      event.preventDefault();
      event.stopImmediatePropagation();
    } else if (Shortcut.ENTER.matches(event)) {
      event.stopImmediatePropagation();
    } else if (Shortcut.isChar(event)) {
      const prefix = this.#buffer.append(event.key);
      const matchedItem = this.items.find((menuItem) =>
        (menuItem.textContent?.trim().toLowerCase() ?? "").startsWith(
          prefix.toLowerCase(),
        ),
      );
      this.focus(matchedItem);
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  }
}

export function setMenuContext(state: MenuContext): MenuContext {
  return setContext<MenuContext>(MenuContext, state);
}

export function getMenuContext(): MenuContext {
  const ctx = getContext<MenuContext>(MenuContext);
  if (!ctx) throw new Error("MenuContext is undefined");
  return ctx;
}
