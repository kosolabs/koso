import { describe, it } from "vitest";
import { Shortcut } from "./shortcuts";

describe("Shortcut tests", () => {
  it("all shortcuts map key name to single character key", () => {
    let property: keyof typeof Shortcut;
    for (property in Shortcut) {
      const maybeShortcut = Shortcut[property];
      if (maybeShortcut instanceof Shortcut) {
        maybeShortcut.toChar();
      }
    }
  });
});
