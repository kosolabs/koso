import { Shortcut } from "$lib/kosui/shortcut";
import { CircleSlash, Icon } from "lucide-svelte";

export const OK = new Shortcut({ key: "Enter" });
export const CANCEL = new Shortcut({ key: "Escape" });
export const INSERT_NODE = new Shortcut({ key: "Enter", shift: true });
export const INSERT_CHILD_NODE = new Shortcut({
  key: "Enter",
  alt: true,
  shift: true,
});

type ActionProps = {
  callback: () => void;
  title?: string;
  description?: string;
  icon?: typeof Icon;
  toolbar?: boolean;
  enabled?: () => boolean;
  shortcut?: Shortcut;
};

export class Action {
  callback: () => void;
  title: string;
  description: string;
  icon: typeof Icon;
  toolbar: boolean;
  enabled: () => boolean;
  shortcut?: Shortcut;

  constructor({
    callback,
    title = "Untitled",
    description,
    icon = CircleSlash,
    toolbar = false,
    enabled = () => true,
    shortcut,
  }: ActionProps) {
    this.callback = callback;
    this.title = title;
    this.description = description || title;
    this.icon = icon;
    this.toolbar = toolbar;
    this.enabled = enabled;
    this.shortcut = shortcut;
  }
}
