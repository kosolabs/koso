import { CircleSlash, type Icon } from "lucide-svelte";
import type { Shortcut } from "../shortcut";

type ActionProps = {
  callback: () => void;
  title?: string;
  description?: string;
  icon?: typeof Icon;
  enabled?: () => boolean;
  shortcut?: Shortcut;
};

export class Action {
  callback: () => void;
  title: string;
  description: string;
  icon: typeof Icon;
  enabled: () => boolean;
  shortcut?: Shortcut;

  constructor({
    callback,
    title = "Untitled",
    description,
    icon = CircleSlash,
    enabled = () => true,
    shortcut,
  }: ActionProps) {
    this.callback = callback;
    this.title = title;
    this.description = description || title;
    this.icon = icon;
    this.enabled = enabled;
    this.shortcut = shortcut;
  }
}
