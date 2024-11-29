import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * Finds the index of the first entry in the iterable that satisfies the
 * provided predicate function.
 *
 * @param entries - An iterable iterator of entries, where each entry is a tuple
 *   containing an index and a value.
 * @param predicate - A function that tests each value, returning true if the
 *   value satisfies the condition, otherwise false.
 * @param missing - The value to return if no entry satisfies the predicate.
 * @returns - The index of the first entry that satisfies the predicate, or the
 *   `missing` value if no entry satisfies the predicate.
 */
export function findEntryIndex<T, U>(
  entries: IterableIterator<[number, T]>,
  predicate: (value: T, index: number) => boolean,
  missing: U,
): number | U {
  for (const [index, value] of entries) {
    if (predicate(value, index)) {
      return index;
    }
  }
  return missing;
}

/**
 * Checks if the given text matches the specified prefix. The match is
 * case-insensitive and can match whole words or prefixes of words.
 *
 * @param text - The text to be checked.
 * @param prefix - The prefix to match against the text.
 * @returns `true` if the text matches the prefix, `false` otherwise.
 */
export function match(text: string, prefix: string): boolean {
  const TOKEN_SPLITTER = /[\s.,!?;:@]/;
  const textLower = text.toLocaleLowerCase();
  const prefixLower = prefix.toLocaleLowerCase();

  if (textLower.startsWith(prefixLower)) {
    return true;
  }

  const words = textLower.split(TOKEN_SPLITTER).filter((w) => w);
  const prefixWords = prefixLower.split(TOKEN_SPLITTER).filter((w) => w);
  for (const prefixWord of prefixWords) {
    let isPrefix = false;
    for (const word of words) {
      if (word.startsWith(prefixWord)) {
        isPrefix = true;
      }
    }
    if (!isPrefix) return false;
  }
  return true;
}
