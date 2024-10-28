import { Set } from "immutable";
import { get } from "svelte/store";
import { beforeEach, describe, expect, it } from "vitest";
import { storable } from "./stores.svelte";

describe("Svelte stores tests", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  describe("storable", () => {
    it("subscribe and unsubscribe work", () => {
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

    it("update changes data and saves to local storage", () => {
      const sw = storable<Set<string>>("vitest-project", Set(), (json) =>
        Set(JSON.parse(json)),
      );
      expect(get(sw)).toEqual(Set());

      sw.update(($sw) => $sw.add("hello"));
      expect(get(sw)).toEqual(Set(["hello"]));

      sw.update(($sw) => $sw.add("world"));
      expect(get(sw)).toEqual(Set(["hello", "world"]));

      const sw2 = storable<Set<string>>("vitest-project", Set(), (json) =>
        Set(JSON.parse(json)),
      );
      expect(get(sw2)).toEqual(Set(["hello", "world"]));
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
