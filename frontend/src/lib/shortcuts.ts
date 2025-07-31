import { Shortcut } from "kosui";

export const OK = new Shortcut({ key: "Enter" });
export const CANCEL = new Shortcut({ key: "Escape" });
export const INSERT_NODE = new Shortcut({ key: "Enter", shift: true });
export const INSERT_CHILD_NODE = new Shortcut({
  key: "Enter",
  alt: true,
  shift: true,
});
