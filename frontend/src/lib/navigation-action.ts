import { goto } from "$app/navigation";
import type { Icon } from "@lucide/svelte";
import type { Shortcut } from "kosui";
import { Action } from "kosui";

type NavigationActionProps = {
  id: string;
  href: string;
  category?: string;
  categoryIndex?: number;
  name?: string;
  index?: number;
  description?: string;
  icon?: typeof Icon;
  shortcut?: Shortcut;
};

export class NavigationAction extends Action {
  #href: string;

  constructor({ id, href, ...restProps }: NavigationActionProps) {
    super({
      id,
      callback: () => goto(href),
      ...restProps,
    });
    this.#href = href;
  }

  get href(): string {
    return this.#href;
  }
}
