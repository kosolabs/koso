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

    it("can save and restore a Set using custom decode", () => {
      const decode = (json: string) => Set<string>(JSON.parse(json));
      const sw = useLocalStorage<Set<string>>("test-key", Set(), { decode });
      sw.value = Set(["a", "b", "c"]);

      const sw2 = useLocalStorage<Set<string>>("test-key", Set(), { decode });
      expect(sw2.value).toEqual(Set(["a", "b", "c"]));
    });

    it("can save and restore a Uint8Array using custom decode and encode", () => {
      const init = new Uint8Array();
      const decode = (json: string) => new Uint8Array(JSON.parse(json));
      const encode = (value: Uint8Array) => JSON.stringify(Array.from(value));

      const sw = useLocalStorage<Uint8Array>("test-key", init, {
        decode,
        encode,
      });
      sw.value = new Uint8Array([1, 2, 3, 4, 5]);

      const sw2 = useLocalStorage<Uint8Array>("test-key", init, {
        decode,
        encode,
      });
      expect(sw2.value).toEqual(new Uint8Array([1, 2, 3, 4, 5]));
    });
  });
});
