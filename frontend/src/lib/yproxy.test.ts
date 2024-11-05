import { describe, expect, it } from "vitest";
import * as Y from "yjs";
import { YChildrenProxy } from "./yproxy";

describe("YChildrenProxy", () => {
  describe("entries", () => {
    const doc = new Y.Doc();
    const data = doc.getArray<string>("data");
    data.push(["a", "b", "c", "d", "e"]);
    const [a, b, c, d, e] = [
      [0, "a"],
      [1, "b"],
      [2, "c"],
      [3, "d"],
      [4, "e"],
    ];

    it("default parameters returns an enumerated version of the input", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries());
      expect(actual).toEqual([a, b, c, d, e]);
    });

    it("[2:] => [c, d, e]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 2 }));
      expect(actual).toEqual([c, d, e]);
    });

    it("[5:] => []", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 5 }));
      expect(actual).toEqual([]);
    });

    it("[-3:] => [c, d, e]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: -3 }));
      expect(actual).toEqual([c, d, e]);
    });

    it("[-6:] => [a, b, c, d, e]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: -6 }));
      expect(actual).toEqual([a, b, c, d, e]);
    });

    it("[0:3] => [a, b, c]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 0, end: 3 }));
      expect(actual).toEqual([a, b, c]);
    });

    it("[2:4] => [c, d]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 2, end: 4 }));
      expect(actual).toEqual([c, d]);
    });

    it("[3:1] => []", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 3, end: 1 }));
      expect(actual).toEqual([]);
    });

    it("[2:-2] => [c]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 2, end: -2 }));
      expect(actual).toEqual([c]);
    });

    it("[2:-6] => []", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 2, end: -6 }));
      expect(actual).toEqual([]);
    });

    it("[2:6] => []", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 2, end: 6 }));
      expect(actual).toEqual([c, d, e]);
    });

    it("[-4:-2] => [b, c]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: -4, end: -2 }));
      expect(actual).toEqual([b, c]);
    });

    it("[::-1] => [e, d, c, b, a]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ step: -1 }));
      expect(actual).toEqual([e, d, c, b, a]);
    });

    it("[2::-1] => [c, b, a]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 2, step: -1 }));
      expect(actual).toEqual([c, b, a]);
    });

    it("[2:0:-1] => [c, b]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: 2, end: 0, step: -1 }),
      );
      expect(actual).toEqual([c, b]);
    });

    it("[-1::-1] => [e, d, c, b, a]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: -1, step: -1 }));
      expect(actual).toEqual([e, d, c, b, a]);
    });

    it("[:-1] => [a, b, c, d]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ end: -1 }));
      expect(actual).toEqual([a, b, c, d]);
    });

    it("[1:4:2] => [b, d]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: 1, end: 4, step: 2 }),
      );
      expect(actual).toEqual([b, d]);
    });

    it("[1:4:-1] => []", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: 1, end: 4, step: -1 }),
      );
      expect(actual).toEqual([]);
    });

    it("[2:-6:-1] => [e, d, c]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: 2, end: -6, step: -1 }),
      );
      expect(actual).toEqual([c, b, a]);
    });

    it("[4:1:-2] => [e, c]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: 4, end: 1, step: -2 }),
      );
      expect(actual).toEqual([e, c]);
    });

    it("[-1:2:-1] => [e, d]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: -1, end: 2, step: -1 }),
      );
      expect(actual).toEqual([e, d]);
    });

    it("[-2:-4:-1] => [d, c]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: -2, end: -4, step: -1 }),
      );
      expect(actual).toEqual([d, c]);
    });

    it("[-1:5:-1] => []", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: -1, end: 5, step: -1 }),
      );
      expect(actual).toEqual([]);
    });

    it("[::-2] => [e, c, a]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ step: -2 }));
      expect(actual).toEqual([e, c, a]);
    });

    it("[::0] throws", () => {
      const children = new YChildrenProxy(data);
      expect(() => Array.from(children.entries({ step: 0 }))).toThrow();
    });
  });
});
