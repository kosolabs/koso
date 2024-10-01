import { Map } from "immutable";
import type { Action } from "./components/ui/command-palette";
import { KeyBinding } from "./key-binding";

export class KeyHandlerRegistry {
  registry: Map<KeyBinding, Action>;

  constructor(actions: Action[]) {
    this.registry = Map<KeyBinding, Action>(
      actions
        .filter((action) => action.shortcut)
        .map((action) => [action.shortcut!, action]),
    );
  }

  handle(event: KeyboardEvent): boolean {
    const action = this.registry.get(KeyBinding.fromEvent(event));
    if (!action || !action.enabled()) return false;
    action.callback();
    event.preventDefault();
    return true;
  }
}
