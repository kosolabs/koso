import type { ClassNameValue } from "tailwind-merge";

export type ElementRef = { ref?: HTMLElement };

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
