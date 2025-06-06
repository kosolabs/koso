import { beforeEach, describe, expect, it, vi } from "vitest";
import * as Y from "yjs";
import { YChildrenProxy, YGraphProxy, YTaskProxy } from "./yproxy";

describe("YTaskProxy", () => {
  let doc: Y.Doc;
  let graph: YGraphProxy;
  let task: YTaskProxy;

  beforeEach(() => {
    doc = new Y.Doc();
    graph = new YGraphProxy(doc.getMap("graph"));
    task = graph.set({
      id: "task-1",
      num: "1",
      name: "Task",
      desc: null,
      children: ["child-1"],
      assignee: null,
      reporter: null,
      status: null,
      statusTime: null,
      kind: null,
      url: null,
      estimate: null,
      deadline: null,
    });
  });

  it("should handle description text operations", () => {
    expect(task.desc).toBeNull();
    task.newDesc();
    expect(task.desc).toBeInstanceOf(Y.Text);
    task.desc?.insert(0, "Description text");
    expect(task.desc?.toString()).toBe("Description text");

    task.delDesc();
    expect(task.desc).toBeNull();
  });

  it("should handle children operations", () => {
    expect(task.children.toArray()).toEqual(["child-1"]);
    task.children.push(["child-2", "child-3"]);
    expect(task.children.toArray()).toEqual(["child-1", "child-2", "child-3"]);

    task.children.delete(1, 1);
    expect(task.children.toArray()).toEqual(["child-1", "child-3"]);

    task.children.replace(["new-child"]);
    expect(task.children.toArray()).toEqual(["new-child"]);
  });

  it("should handle estimate operations", () => {
    expect(task.estimate).toBeNull();
    task.estimate = 1;
    expect(task.estimate).toStrictEqual(1);
    task.estimate = 13;
    expect(task.estimate).toStrictEqual(13);
  });

  it("should handle deadline operations", () => {
    expect(task.deadline).toBeNull();
    task.deadline = 0;
    expect(task.deadline).toBeNull();
    task.deadline = 1;
    expect(task.deadline).toStrictEqual(1);
    const now = Date.now();
    task.deadline = now;
    expect(task.deadline).toStrictEqual(now);
  });

  it("should handle subscribe/unsubscribe functionality", () => {
    const changes: string[] = [];
    const unsubscribe = task.subscribe((value) => {
      changes.push(value.name);
    });

    expect(changes).toEqual(["Task"]); // Initial call

    task.name = "Updated Task";
    expect(changes).toEqual(["Task", "Updated Task"]);

    task.children.push(["child-1"]);
    expect(changes).toEqual(["Task", "Updated Task", "Updated Task"]);

    unsubscribe();
    task.name = "Final Task";
    expect(changes).toEqual(["Task", "Updated Task", "Updated Task"]); // No new updates after unsubscribe
  });

  it("should handle multiple subscribers correctly", () => {
    const changes1: string[] = [];
    const changes2: string[] = [];

    const unsubscribe1 = task.subscribe((value) => {
      changes1.push(value.name);
    });

    const unsubscribe2 = task.subscribe((value) => {
      changes2.push(value.name);
    });

    expect(changes1).toEqual(["Task"]);
    expect(changes2).toEqual(["Task"]);

    task.name = "Updated Task";
    expect(changes1).toEqual(["Task", "Updated Task"]);
    expect(changes2).toEqual(["Task", "Updated Task"]);

    unsubscribe1();
    task.name = "Final Task";
    expect(changes1).toEqual(["Task", "Updated Task"]); // No update after unsubscribe
    expect(changes2).toEqual(["Task", "Updated Task", "Final Task"]);

    unsubscribe2();
    task.name = "Very Final Task";
    expect(changes1).toEqual(["Task", "Updated Task"]); // No update
    expect(changes2).toEqual(["Task", "Updated Task", "Final Task"]); // No update
  });

  it("should call unobserveDeep when last subscriber unsubscribes", () => {
    // Spy on the unobserveDeep method
    const unobserveDeepSpy = vi.spyOn(task, "unobserveDeep");

    // Add two subscribers
    const unsubscribe1 = task.subscribe(() => {});
    const unsubscribe2 = task.subscribe(() => {});

    // First unsubscribe should not trigger unobserveDeep
    unsubscribe1();
    expect(unobserveDeepSpy).not.toHaveBeenCalled();

    // Last unsubscribe should trigger unobserveDeep
    unsubscribe2();
    expect(unobserveDeepSpy).toHaveBeenCalledTimes(1);
  });

  it("should handle subscribe/unsubscribe cycle correctly", () => {
    const observeDeepSpy = vi.spyOn(task, "observeDeep");
    const unobserveDeepSpy = vi.spyOn(task, "unobserveDeep");

    // First subscription
    const unsubscribe1 = task.subscribe(() => {});
    expect(observeDeepSpy).toHaveBeenCalledTimes(1);

    // Second subscription
    const unsubscribe2 = task.subscribe(() => {});
    expect(observeDeepSpy).toHaveBeenCalledTimes(1); // Should not call observeDeep again

    // First unsubscribe
    unsubscribe1();
    expect(unobserveDeepSpy).not.toHaveBeenCalled();

    // Last unsubscribe
    unsubscribe2();
    expect(unobserveDeepSpy).toHaveBeenCalledTimes(1);

    // New subscription after all unsubscribed
    const unsubscribe3 = task.subscribe(() => {});
    expect(observeDeepSpy).toHaveBeenCalledTimes(2); // Should call observeDeep again

    unsubscribe3();
    expect(unobserveDeepSpy).toHaveBeenCalledTimes(2);
  });
});

describe("YChildrenProxy", () => {
  describe("entries", () => {
    const doc = new Y.Doc();
    const data = doc.getArray<string>("data");
    data.push(["a", "b", "c", "d", "e"]);
    const [a, b, c, d, e] = [
      [0, "a"],
      [1, "b"],
      [2, "c"],
      [3, "d"],
      [4, "e"],
    ];

    it("default parameters returns an enumerated version of the input", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries());
      expect(actual).toEqual([a, b, c, d, e]);
    });

    it("[2:] => [c, d, e]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 2 }));
      expect(actual).toEqual([c, d, e]);
    });

    it("[5:] => []", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 5 }));
      expect(actual).toEqual([]);
    });

    it("[-3:] => [c, d, e]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: -3 }));
      expect(actual).toEqual([c, d, e]);
    });

    it("[-6:] => [a, b, c, d, e]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: -6 }));
      expect(actual).toEqual([a, b, c, d, e]);
    });

    it("[0:3] => [a, b, c]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 0, end: 3 }));
      expect(actual).toEqual([a, b, c]);
    });

    it("[2:4] => [c, d]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 2, end: 4 }));
      expect(actual).toEqual([c, d]);
    });

    it("[3:1] => []", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 3, end: 1 }));
      expect(actual).toEqual([]);
    });

    it("[2:-2] => [c]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 2, end: -2 }));
      expect(actual).toEqual([c]);
    });

    it("[2:-6] => []", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 2, end: -6 }));
      expect(actual).toEqual([]);
    });

    it("[2:6] => []", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 2, end: 6 }));
      expect(actual).toEqual([c, d, e]);
    });

    it("[-4:-2] => [b, c]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: -4, end: -2 }));
      expect(actual).toEqual([b, c]);
    });

    it("[::-1] => [e, d, c, b, a]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ step: -1 }));
      expect(actual).toEqual([e, d, c, b, a]);
    });

    it("[2::-1] => [c, b, a]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: 2, step: -1 }));
      expect(actual).toEqual([c, b, a]);
    });

    it("[2:0:-1] => [c, b]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: 2, end: 0, step: -1 }),
      );
      expect(actual).toEqual([c, b]);
    });

    it("[-1::-1] => [e, d, c, b, a]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ start: -1, step: -1 }));
      expect(actual).toEqual([e, d, c, b, a]);
    });

    it("[:-1] => [a, b, c, d]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ end: -1 }));
      expect(actual).toEqual([a, b, c, d]);
    });

    it("[1:4:2] => [b, d]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: 1, end: 4, step: 2 }),
      );
      expect(actual).toEqual([b, d]);
    });

    it("[1:4:-1] => []", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: 1, end: 4, step: -1 }),
      );
      expect(actual).toEqual([]);
    });

    it("[2:-6:-1] => [e, d, c]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: 2, end: -6, step: -1 }),
      );
      expect(actual).toEqual([c, b, a]);
    });

    it("[4:1:-2] => [e, c]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: 4, end: 1, step: -2 }),
      );
      expect(actual).toEqual([e, c]);
    });

    it("[-1:2:-1] => [e, d]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: -1, end: 2, step: -1 }),
      );
      expect(actual).toEqual([e, d]);
    });

    it("[-2:-4:-1] => [d, c]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: -2, end: -4, step: -1 }),
      );
      expect(actual).toEqual([d, c]);
    });

    it("[-1:5:-1] => []", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(
        children.entries({ start: -1, end: 5, step: -1 }),
      );
      expect(actual).toEqual([]);
    });

    it("[::-2] => [e, c, a]", () => {
      const children = new YChildrenProxy(data);
      const actual = Array.from(children.entries({ step: -2 }));
      expect(actual).toEqual([e, c, a]);
    });

    it("[::0] throws", () => {
      const children = new YChildrenProxy(data);
      expect(() => Array.from(children.entries({ step: 0 }))).toThrow();
    });
  });
});
