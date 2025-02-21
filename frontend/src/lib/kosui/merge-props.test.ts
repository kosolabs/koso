import { describe, expect, it, vi } from "vitest";
import { mergeProps } from "./merge-props";

class CustomEvent extends Event {
  defaultPrevented = false;

  preventDefault() {
    this.defaultPrevented = true;
  }
}

describe("mergeProps", () => {
  it("should merge basic props", () => {
    const props1 = { a: 1, b: 2 };
    const props2 = { c: 3 };

    const actual = mergeProps(props1, props2);

    expect(actual).toEqual({ a: 1, b: 2, c: 3 });
  });

  it("should override earlier props from latter", () => {
    const props1 = { a: 1 };
    const props2 = { a: 2 };

    const actual = mergeProps(props1, props2);

    expect(actual).toEqual({ a: 2 });
  });

  it("should chain event handlers", () => {
    const handler1 = vi.fn();
    const handler2 = vi.fn();
    const props1 = { onclick: handler1 };
    const props2 = { onclick: handler2 };
    const result = mergeProps(props1, props2);

    result.onclick(new CustomEvent("click"));

    expect(handler1).toHaveBeenCalled();
    expect(handler2).toHaveBeenCalled();
  });

  it("should not run second handler if the first once cancels", () => {
    const handler1 = vi.fn((event) => event.stopImmediatePropagation());
    const handler2 = vi.fn();
    const props1 = { onclick: handler1 };
    const props2 = { onclick: handler2 };
    const result = mergeProps(props1, props2);

    result.onclick(new CustomEvent("click"));

    expect(handler1).toHaveBeenCalled();
    expect(handler2).not.toHaveBeenCalled();
  });

  it("should chain functions", () => {
    const values: number[] = [];
    const props1 = {
      callback: (num1: number, num2: number) => values.push(num1 + num2),
    };
    const props2 = {
      callback: (num1: number, num2: number) => values.push(num1 * num2),
    };
    const result = mergeProps(props1, props2);

    result.callback(5, 5);

    expect(values).toEqual([10, 25]);
  });

  it("should merge class names", () => {
    const props1 = { class: "foo" };
    const props2 = { class: "bar" };
    const result = mergeProps(props1, props2);
    expect(result).toEqual({ class: "foo bar" });
  });

  it("should dedupe tailwind classes", () => {
    const props1 = { class: "text-lg" };
    const props2 = { class: "text-sm" };
    const result = mergeProps(props1, props2);
    expect(result).toEqual({ class: "text-sm" });
  });
});
