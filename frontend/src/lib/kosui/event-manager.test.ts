import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { EventManager } from "./event-manager";

describe("EventManager tests", () => {
  let events: EventManager;
  beforeEach(() => {
    vi.resetAllMocks();
    events = new EventManager();
  });

  afterEach(() => {
    events.destroy();
  });

  it("a single event is registered, that event's listener is called", () => {
    const calls: string[] = [];
    events.on("keydown", () => calls.push("listener1"));

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "a" }));

    expect(calls).toStrictEqual(["listener1"]);
  });

  it("listeners are called in reverse order", () => {
    const calls: string[] = [];
    events.on("keydown", () => calls.push("listener1"));
    events.on("keydown", () => calls.push("listener2"));

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "a" }));

    expect(calls).toStrictEqual(["listener2", "listener1"]);
  });

  it("only the first listener is called when stopImmediatePropagation is called", () => {
    const calls: string[] = [];
    events.on("keydown", () => calls.push("listener1"));
    events.on("keydown", (event) => {
      event.stopImmediatePropagation();
      calls.push("listener2");
    });

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "a" }));

    expect(calls).toStrictEqual(["listener2"]);
  });

  it("listeners can unregistered and will not be called", () => {
    const calls: string[] = [];
    events.on("keydown", () => calls.push("listener1"));
    const unregister = events.on("keydown", () => calls.push("listener2"));
    unregister();

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "a" }));

    expect(calls).toStrictEqual(["listener1"]);
  });

  it("unregistering all listeners does not leak listeners", () => {
    const addEventListenerSpy = vi.spyOn(document, "addEventListener");
    const removeEventListenerSpy = vi.spyOn(document, "removeEventListener");

    const unregister1 = events.on("keydown", () => {});
    const unregister2 = events.on("keydown", () => {});
    expect(addEventListenerSpy).toHaveBeenCalledOnce();

    unregister1();
    expect(removeEventListenerSpy).not.toHaveBeenCalled();

    unregister2();
    expect(removeEventListenerSpy).toHaveBeenCalledOnce();
  });

  it("destroy removes all listeners", () => {
    const addEventListenerSpy = vi.spyOn(document, "addEventListener");
    const removeEventListenerSpy = vi.spyOn(document, "removeEventListener");

    events.on("keydown", () => {});
    events.on("keydown", () => {});
    expect(addEventListenerSpy).toHaveBeenCalledOnce();
    expect(removeEventListenerSpy).not.toHaveBeenCalled();

    events.destroy();
    expect(removeEventListenerSpy).toHaveBeenCalledOnce();
  });
});
