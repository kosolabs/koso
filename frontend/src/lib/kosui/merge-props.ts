import type { Component, ComponentProps } from "svelte";
import type { EventHandler } from "svelte/elements";
import { twMerge, type ClassNameValue } from "tailwind-merge";

function isClass(value: unknown): value is ClassNameValue {
  if (value === null || value === undefined) {
    return true;
  }
  if (["string", "number", "bigint", "boolean"].includes(typeof value)) {
    return true;
  }
  if (Array.isArray(value)) {
    return value.every((item) => isClass(item));
  }
  return false;
}

function isHandlerKey(key: string): boolean {
  return key.startsWith("on") && key[2].toLowerCase() === key[2];
}

function isHandler(value: unknown): value is EventHandler {
  return typeof value === "function";
}

export function interceptStopImmediatePropagation(
  event: Event,
  callback: () => void,
) {
  const original = event.stopImmediatePropagation;

  event.stopImmediatePropagation = function () {
    callback();
    original.call(event);
  };
}

export function runEventHandlers(
  target: EventTarget,
  event: Event & { currentTarget: EventTarget & Element },
  ...handlers: EventHandler[]
) {
  let stopped = false;
  interceptStopImmediatePropagation(event, () => (stopped = true));
  for (const handler of handlers) {
    if (stopped) break;
    handler.call(target, event);
  }
}

function chainHandlers<
  E extends Event & { currentTarget: EventTarget & Element },
  T extends EventTarget,
>(...handlers: EventHandler[]): (e: E) => void {
  return function (this: T, event: E) {
    runEventHandlers(this, event, ...handlers);
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

      if (key === "class" && isClass(prevVal) && isClass(currVal)) {
        result[key] = twMerge(prevVal, currVal);
        continue;
      }

      result[key] = currVal;
    }
  }

  return result as PropsUnion<T>;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function mergeComponentProps<C extends Component<any>>(
  _component: C,
  ...args: Partial<ComponentProps<C>>[]
): Partial<ComponentProps<C>> {
  return mergeProps(...args) as Partial<ComponentProps<C>>;
}
