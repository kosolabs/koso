import { beforeEach } from "node:test";
import { get, writable } from "svelte/store";
import { describe, expect, it } from "vitest";
import { storedWritable } from "./stores";

describe("Svelte stores tests", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  describe("storedWritable", () => {
    it("store contract is adhered to and subscribe and unsubscribe work", () => {
      const project = writable("project1");
      const sw = storedWritable<string>("vitest-", project, "hello");
      const values: string[] = [];
      const unsubscribe = sw.subscribe((value) => values.push(value));

      expect(values).toEqual(["hello"]);

      sw.set("world");
      expect(values).toEqual(["hello", "world"]);

      unsubscribe();

      sw.set("other");
      expect(get(sw)).toEqual("world");
      expect(values).toEqual(["hello", "world"]);
    });

    it("creating a new store loads previous data", () => {
      const project = writable("project1");
      const sw = storedWritable<string>("vitest-", project, "hello");

      const unsubscribe = sw.subscribe(() => {});
      sw.set("world");
      unsubscribe();

      const sw2 = storedWritable<string>("vitest-", project, "hello");
      const unsubscribe2 = sw2.subscribe(() => {});
      expect(get(sw2)).toEqual("world");
      unsubscribe2();
    });

    it("switching scope reloads the corresponding store", () => {
      const project = writable("default");
      const sw = storedWritable<string>("vitest-", project, "default");
      const unsubscribe = sw.subscribe(() => {});

      project.set("project1");
      sw.set("value1");

      project.set("project2");
      sw.set("value2");

      project.set("project1");
      expect(get(sw)).toEqual("value1");

      project.set("project2");
      expect(get(sw)).toEqual("value2");

      unsubscribe();
    });
  });
});
