import { Map } from "immutable";
import { KeyBinding } from "./key-binding";

export type Handler = () => void;

export class KeyHandlerRegistry {
  registry: Map<KeyBinding, Handler>;

  constructor(registry: [KeyBinding, Handler][]) {
    this.registry = Map<KeyBinding, Handler>(registry);
  }

  handle(event: KeyboardEvent) {
    const handler = this.registry.get(KeyBinding.fromEvent(event));
    if (!handler) return;
    handler();
    event.preventDefault();
    event.stopPropagation();
  }
}
