import type { EventHandler } from "svelte/elements";
import { twMerge } from "tailwind-merge";
import type { ClassValue } from "tailwind-variants";

function isClassValue(value: unknown): value is ClassValue {
  if (value === null || value === undefined) {
    return true;
  }
  if (["string", "number", "bigint", "boolean"].includes(typeof value)) {
    return true;
  }
  if (Array.isArray(value)) {
    return value.every((item) => isClassValue(item));
  }
  return false;
}

function isHandlerKey(key: string): boolean {
  return key.startsWith("on") && key[2].toLowerCase() === key[2];
}

function isHandler(value: unknown): value is EventHandler {
  return typeof value === "function";
}

function chainHandlers<
  E extends Event & { currentTarget: EventTarget & Element },
  T extends EventTarget,
>(...handlers: EventHandler[]): (e: E) => void {
  return function (this: T, event: E) {
    for (const handler of handlers) {
      if (event.defaultPrevented) break;
      handler.call(this, event);
    }
  };
}

type Callable = (...args: unknown[]) => void;

function isCallback(value: unknown): value is Callable {
  return typeof value === "function";
}

function chainCallbacks(...callbacks: Callable[]): Callable {
  return function (...args: unknown[]) {
    for (const callback of callbacks) {
      callback(...args);
    }
  };
}

type Props = Record<string, unknown>;

type PropsUnion<T extends Props[]> = (
  T extends (infer U)[]
    ? U extends Props
      ? (k: { [K in keyof U]: U[K] }) => void
      : never
    : never
) extends (k: infer I) => void
  ? I
  : never;

export function mergeProps<T extends Props[]>(...args: [...T]): PropsUnion<T> {
  const result: Props = {};
  for (const props of args) {
    for (const key in props) {
      const prevVal = result[key];
      const currVal = props[key];

      if (currVal === undefined) {
        continue;
      }

      if (prevVal === undefined) {
        result[key] = currVal;
        continue;
      }

      if (isHandlerKey(key) && isHandler(prevVal) && isHandler(currVal)) {
        result[key] = chainHandlers(prevVal, currVal);
        continue;
      }

      if (isCallback(prevVal) && isCallback(currVal)) {
        result[key] = chainCallbacks(prevVal, currVal);
        continue;
      }

      if (key === "class" && isClassValue(prevVal) && isClassValue(currVal)) {
        result[key] = twMerge(prevVal, currVal);
        continue;
      }

      result[key] = currVal;
    }
  }

  return result as PropsUnion<T>;
}
