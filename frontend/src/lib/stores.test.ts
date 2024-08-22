import { beforeEach } from "node:test";
import { get } from "svelte/store";
import { describe, expect, it } from "vitest";
import { storable } from "./stores";

describe("Svelte stores tests", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  describe("storable", () => {
    it("store contract is adhered to and subscribe and unsubscribe work", () => {
      const sw = storable<string>("vitest-project", "hello");
      const values: string[] = [];
      const unsubscribe = sw.subscribe((value) => values.push(value));

      expect(values).toEqual(["hello"]);

      sw.set("world");
      expect(values).toEqual(["hello", "world"]);

      unsubscribe();

      sw.set("other");
      expect(values).toEqual(["hello", "world"]);
    });

    it("creating a new store loads previous data", () => {
      const sw = storable<string>("vitest-project1", "hello");

      const unsubscribe = sw.subscribe(() => {});
      sw.set("world");
      unsubscribe();

      const sw2 = storable<string>("vitest-project1", "hello");
      const unsubscribe2 = sw2.subscribe(() => {});
      expect(get(sw2)).toEqual("world");
      unsubscribe2();
    });
  });
});
