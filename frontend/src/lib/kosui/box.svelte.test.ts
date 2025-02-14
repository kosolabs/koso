import { describe, expect, it } from "vitest";
import { Box } from "./box.svelte";

describe("Box tests", () => {
  it("getters and setters work", () => {
    const box = new Box<string>();
    box.value = "koso";
    expect(box.value).toEqual("koso");
  });

  it("apply works", () => {
    const box = new Box<string>();
    box.apply("koso");
    expect(box.value).toEqual("koso");
  });
});
