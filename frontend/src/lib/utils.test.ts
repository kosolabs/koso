import { describe, expect, it } from "vitest";
import { findEntryIndex, match } from "./utils";

describe("utils tests", () => {
  describe("match", () => {
    it("equality matches", () => {
      expect(
        match("Prototype Koso notifications", "prototype koso notifications"),
      ).toBeTruthy();
    });

    it("full prefix matches", () => {
      expect(
        match("Prototype Koso notifications", "prototype koso"),
      ).toBeTruthy();
    });

    it("non-prefix substring does not match", () => {
      expect(match("Prototype Koso notifications", "type")).toBeFalsy();
    });

    it("prefix of a word matches", () => {
      expect(match("Prototype Koso notifications", "ko")).toBeTruthy();
    });

    it("multiple prefixes of multiple words matches", () => {
      expect(match("Prototype Koso notifications", "ko pro")).toBeTruthy();
    });

    it("equality of email matches", () => {
      expect(match("my.email@gmail.com", "my.email@gmail.com")).toBeTruthy();
    });

    it("full prefix of email matches", () => {
      expect(match("my.email@gmail.com", "my.email")).toBeTruthy();
    });

    it("non-prefix substring of email does not match", () => {
      expect(match("my.email@gmail.com", "mail")).toBeFalsy();
    });

    it("prefix of a word in an email matches", () => {
      expect(match("my.email@gmail.com", "email")).toBeTruthy();
    });

    it("multiple prefixes of multiple words in an email matches", () => {
      expect(match("my.email@gmail.com", "gmail email my")).toBeTruthy();
    });

    it("prefix of title following task tag matches", () => {
      expect(match("Reliability:Break everything", "Break")).toBeTruthy();
    });
  });

  describe("findEntryIndex", () => {
    it("finds the index of the first matching entry", () => {
      const entries = ["apple", "banana", "cherry"].entries();
      const predicate = (value: string) => value.startsWith("b");
      expect(findEntryIndex(entries, predicate, -1)).toBe(1);
    });

    it("returns the missing value if no entry matches", () => {
      const entries = ["apple", "banana", "cherry"].entries();
      const predicate = (value: string) => value.startsWith("d");
      expect(findEntryIndex(entries, predicate, -1)).toBe(-1);
    });

    it("works with different types of values", () => {
      const entries = [10, 20, 30].entries();
      const predicate = (value: number) => value > 15;
      expect(findEntryIndex(entries, predicate, -1)).toBe(1);
    });

    it("returns the first matching index when multiple entries match", () => {
      const entries = ["apple", "banana", "cherry"].entries();
      const predicate = (value: string) => value.startsWith("b");
      expect(findEntryIndex(entries, predicate, -1)).toBe(1);
    });

    it("handles empty iterables", () => {
      const entries = ["apple", "banana", "cherry"].entries();
      const predicate = (value: string) => value.startsWith("d");
      expect(findEntryIndex(entries, predicate, -1)).toBe(-1);
    });
  });
});
