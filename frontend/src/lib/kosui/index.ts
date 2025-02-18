import { EventManager } from "./event-manager";

/**
 * A global event manager. Register event listeners with the `on` method.
 * Listeners will execute in LIFO order. Any listener that calls
 * `stopImmediatePropagation()` will cause the event manager to stop processing
 * subsequent listeners.
 *
 * @example
 *   events.on("keydown", (event) => {
 *     console.log("keydown event:", event);
 *   });
 */
export const events = new EventManager();
