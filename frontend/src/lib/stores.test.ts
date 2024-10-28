import { Set } from "immutable";
import { get } from "svelte/store";
import { beforeEach, describe, expect, it } from "vitest";
import { storable, useLocalStorage } from "./stores.svelte";

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

  describe("useLocalStorage", () => {
    it("can save and restore a string from local storage", () => {
      const sw = useLocalStorage("test-key", "hello");
      sw.value = "world";

      const sw2 = useLocalStorage("test-key", "hello");
      expect(sw2.value).toEqual("world");
    });

    it("can save and restore an object from local storage", () => {
      const sw = useLocalStorage("test-key", {});
      sw.value = { name: "koso", value: 123 };

      const sw2 = useLocalStorage("test-key", {});
      expect(sw2.value).toEqual({ name: "koso", value: 123 });
    });

    it("can save and restore a Set using custom parser", () => {
      const sw = useLocalStorage<Set<string>>("test-key", Set(), (json) =>
        Set(JSON.parse(json)),
      );
      sw.value = Set(["a", "b", "c"]);

      const sw2 = useLocalStorage<Set<string>>("test-key", Set(), (json) =>
        Set(JSON.parse(json)),
      );
      expect(sw2.value).toEqual(Set(["a", "b", "c"]));
    });

    it("can save and restore a Uint8Array using custom parser and serializer", () => {
      const init = new Uint8Array();
      const parser = (json: string) => new Uint8Array(JSON.parse(json));
      const serializer = (value: Uint8Array) =>
        JSON.stringify(Array.from(value));

      const sw = useLocalStorage<Uint8Array>(
        "test-key",
        init,
        parser,
        serializer,
      );
      sw.value = new Uint8Array([1, 2, 3, 4, 5]);

      const sw2 = useLocalStorage<Uint8Array>(
        "test-key",
        init,
        parser,
        serializer,
      );
      expect(sw2.value).toEqual(new Uint8Array([1, 2, 3, 4, 5]));
    });
  });
});
