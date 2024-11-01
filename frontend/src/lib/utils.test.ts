import { describe, expect, it } from "vitest";
import { match } from "./utils";

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
  });
});
