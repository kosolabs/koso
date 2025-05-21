import { goto } from "$app/navigation";
import { page } from "$app/state";
import type { Icon } from "lucide-svelte";
import type { ActionID } from "./components/ui/command-palette";
import { Action } from "./kosui/command";
import type { Shortcut } from "./kosui/shortcut";

type NavigationActionProps = {
  id: ActionID;
  href: string;
  title?: string;
  description?: string;
  icon?: typeof Icon;
  shortcut?: Shortcut;
};

export class NavigationAction extends Action<ActionID> {
  #href: string;

  constructor({ id, href, ...restProps }: NavigationActionProps) {
    super({
      id,
      callback: () => goto(href),
      enabled: () => page.url.pathname !== href,
      ...restProps,
    });
    this.#href = href;
  }

  get href(): string {
    return this.#href;
  }
}
