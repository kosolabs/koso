import { Map } from "immutable";
import { KeyBinding } from "./key-binding";

export type Handler = () => void;

export class KeyHandlerRegistry {
  registry: Map<KeyBinding, Handler>;

  constructor(registry: [KeyBinding, Handler][]) {
    this.registry = Map<KeyBinding, Handler>(registry);
  }

  handle(event: KeyboardEvent): boolean {
    const handler = this.registry.get(KeyBinding.fromEvent(event));
    if (!handler) return false;
    handler();
    event.preventDefault();
    return true;
  }
}
