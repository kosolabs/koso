import { Set } from "immutable";
import { beforeEach, describe, expect, it } from "vitest";
import { useLocalStorage } from "./stores.svelte";

describe("Svelte stores tests", () => {
  beforeEach(() => {
    localStorage.clear();
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
