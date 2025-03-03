import type { ClassNameValue } from "tailwind-merge";

export function noop() {}

export type ElementRef = {
  el?: HTMLElement;
  ref?: (el: HTMLElement) => void;
};

export type ClassName = { class?: ClassNameValue };

export type ToggleEventWithTarget<T extends HTMLElement> = ToggleEvent & {
  currentTarget: EventTarget & T;
};

/**
 * Converts a kebab-case string to Title Case.
 *
 * @example
 *   toTitleCase("hello-world"); // returns "Hello World"
 *
 * @param kebab - The kebab-case string to convert
 * @returns The string converted to Title Case with spaces between words
 */
export function toTitleCase(kebab: string) {
  return kebab
    .split("-")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

export class TypingBuffer {
  #prefix: string = "";
  #timer: ReturnType<typeof setTimeout> | undefined;
  #timeout: number = 500;

  constructor(timeout?: number) {
    if (timeout !== undefined) {
      this.#timeout = timeout;
    }
  }

  get prefix(): string {
    return this.#prefix;
  }

  append(char: string): string {
    clearTimeout(this.#timer);
    this.#prefix += char;
    this.#timer = setTimeout(() => (this.#prefix = ""), this.#timeout);
    return this.prefix;
  }
}
