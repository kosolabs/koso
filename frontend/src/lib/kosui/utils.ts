export type ElementRef = { ref?: HTMLElement };

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
