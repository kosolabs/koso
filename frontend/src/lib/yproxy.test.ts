import { describe, expect, it } from "vitest";
import * as Y from "yjs";
import { YChildrenProxy, YGraphProxy, type YGraph } from "./yproxy";

describe("YTaskProxy", () => {
  it("should create YTaskProxy correctly", () => {
    const doc = new Y.Doc();
    const yMap = doc.getMap("task");
    yMap.set("id", "task-1");
    yMap.set("num", "1");
    yMap.set("name", "Test Task");
    yMap.set("children", Y.Array.from(["child-1", "child-2"]));

    const taskProxy = new YGraphProxy(doc.getMap("graph")).set({
      id: "task-1",
      num: "1",
      name: "Test Task",
      desc: null,
      children: ["child-1", "child-2"],
      assignee: null,
      reporter: null,
      status: null,
      statusTime: null,
      kind: null,
      url: null,
    });

    expect(taskProxy.id).toBe("task-1");
    expect(taskProxy.num).toBe("1");
    expect(taskProxy.name).toBe("Test Task");
    expect(taskProxy.children.toArray()).toEqual(["child-1", "child-2"]);
    expect(taskProxy.desc).toBeNull();
    expect(taskProxy.assignee).toBeNull();
    expect(taskProxy.reporter).toBeNull();
    expect(taskProxy.yStatus).toBeNull();
    expect(taskProxy.statusTime).toBeNull();
    expect(taskProxy.yKind).toBeNull();
    expect(taskProxy.url).toBeNull();
  });

  it("should handle property updates correctly", () => {
    const doc = new Y.Doc();
    const yGraph: YGraph = doc.getMap("graph");
    const graphProxy = new YGraphProxy(yGraph);

    const task = graphProxy.set({
      id: "task-1",
      num: "1",
      name: "Initial Name",
      desc: null,
      children: [],
      assignee: null,
      reporter: null,
      status: null,
      statusTime: null,
      kind: null,
      url: null,
    });

    task.name = "Updated Name";
    task.assignee = "user1";
    task.yStatus = "In Progress";
    task.statusTime = 1234567890;

    expect(task.name).toBe("Updated Name");
    expect(task.assignee).toBe("user1");
    expect(task.yStatus).toBe("In Progress");
    expect(task.statusTime).toBe(1234567890);
  });

  it("should handle description text operations", () => {
    const doc = new Y.Doc();
    const yGraph: YGraph = doc.getMap("graph");
    const graphProxy = new YGraphProxy(yGraph);

    const task = graphProxy.set({
      id: "task-1",
      num: "1",
      name: "Task",
      desc: null,
      children: [],
      assignee: null,
      reporter: null,
      status: null,
      statusTime: null,
      kind: null,
      url: null,
    });

    expect(task.desc).toBeNull();
    task.newDesc();
    expect(task.desc).toBeInstanceOf(Y.Text);
    task.desc?.insert(0, "Description text");
    expect(task.desc?.toString()).toBe("Description text");

    task.delDesc();
    expect(task.desc).toBeNull();
  });

  it("should handle children operations", () => {
    const doc = new Y.Doc();
    const yGraph: YGraph = doc.getMap("graph");
    const graphProxy = new YGraphProxy(yGraph);

    const task = graphProxy.set({
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
    });

    expect(task.children.toArray()).toEqual(["child-1"]);
    task.children.push(["child-2", "child-3"]);
    expect(task.children.toArray()).toEqual(["child-1", "child-2", "child-3"]);

    task.children.delete(1, 1);
    expect(task.children.toArray()).toEqual(["child-1", "child-3"]);

    task.children.replace(["new-child"]);
    expect(task.children.toArray()).toEqual(["new-child"]);
  });

  it("should handle subscribe/unsubscribe functionality", () => {
    const doc = new Y.Doc();
    const yGraph: YGraph = doc.getMap("graph");
    const graphProxy = new YGraphProxy(yGraph);

    const task = graphProxy.set({
      id: "task-1",
      num: "1",
      name: "Task",
      desc: null,
      children: [],
      assignee: null,
      reporter: null,
      status: null,
      statusTime: null,
      kind: null,
      url: null,
    });

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
    const doc = new Y.Doc();
    const yGraph: YGraph = doc.getMap("graph");
    const graphProxy = new YGraphProxy(yGraph);

    const task = graphProxy.set({
      id: "task-1",
      num: "1",
      name: "Task",
      desc: null,
      children: [],
      assignee: null,
      reporter: null,
      status: null,
      statusTime: null,
      kind: null,
      url: null,
    });

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
