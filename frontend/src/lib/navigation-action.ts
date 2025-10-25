import { goto } from "$app/navigation";
import type { ResolvedPathname } from "$app/types";
import type { Icon } from "@lucide/svelte";
import type { Shortcut } from "kosui";
import { Action } from "kosui";

type NavigationActionProps = {
  id: string;
  href: ResolvedPathname;
  category?: string;
  categoryIndex?: number;
  name?: string;
  index?: number;
  description?: string;
  icon?: typeof Icon;
  shortcut?: Shortcut;
};

export class NavigationAction extends Action {
  #href: ResolvedPathname;

  constructor({ id, href, ...restProps }: NavigationActionProps) {
    super({
      id,
      callback: () => goto(href),
      ...restProps,
    });
    this.#href = href;
  }

  get href(): ResolvedPathname {
    return this.#href;
  }
}
