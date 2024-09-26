import { Map } from "immutable";
import { KeyBinding } from "./key-binding";

export type Handler = { func: () => void; bubble?: boolean };

export class KeyHandlerRegistry {
  registry: Map<KeyBinding, Handler>;

  constructor(registry: [KeyBinding, Handler][]) {
    this.registry = Map<KeyBinding, Handler>(registry);
  }

  handle(event: KeyboardEvent): boolean {
    const handler = this.registry.get(KeyBinding.fromEvent(event));
    if (!handler) return false;
    const { func, bubble = false } = handler;
    func();
    event.preventDefault();
    if (!bubble) event.stopPropagation();
    return true;
  }
}
