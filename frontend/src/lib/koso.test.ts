import { List, Set } from "immutable";
import * as encoding from "lib0/encoding";
import { beforeEach, describe, expect, it } from "vitest";
import * as Y from "yjs";
import { type TaskBuilder } from "../../tests/utils";
import type { User } from "./auth.svelte";
import { Koso, Node } from "./koso.svelte";

const USER: User = {
  email: "t@koso.app",
  name: "Test User",
  picture: "",
  exp: 0,
};

const OTHER_USER: User = {
  email: "t2@koso.app",
  name: "Test2 User",
  picture: "",
  exp: 0,
};

const EMPTY_SYNC_RESPONSE = (() => {
  const encoder = encoding.createEncoder();
  encoding.writeVarUint(encoder, 0);
  encoding.writeVarUint(encoder, 1);
  encoding.writeVarUint8Array(encoder, Y.encodeStateAsUpdateV2(new Y.Doc()));
  return encoding.toUint8Array(encoder);
})();

describe("Koso tests", () => {
  const root = new Node();
  let koso: Koso;

  const init = (tasks: TaskBuilder[]) => {
    const upsertedTaskIds = Set<string>(tasks.map((t) => t.id));
    const childTaskIds = Set<string>(tasks.flatMap((t) => t.children ?? []));
    const remainingTaskIds = childTaskIds.subtract(upsertedTaskIds);
    koso.doc.transact(() => {
      for (const task of tasks) {
        koso.upsert({
          id: task.id,
          num: task.num ?? task.id,
          name: task.name ?? `Task ${task.id}`,
          children: task.children ?? [],
          assignee: task.assignee ?? null,
          reporter: task.reporter ?? null,
          status: task.status ?? null,
          statusTime: task.statusTime ?? null,
        });
      }
      for (const taskId of remainingTaskIds) {
        koso.upsert({
          id: taskId,
          num: taskId,
          name: `Task ${taskId}`,
          children: [],
          assignee: null,
          reporter: null,
          status: null,
          statusTime: null,
        });
      }
    });
  };

  beforeEach(() => {
    koso = new Koso("project-id", new Y.Doc());
    koso.handleClientMessage(() => {});
    koso.handleServerMessage(EMPTY_SYNC_RESPONSE);
  });

  describe("insertNode", () => {
    it("creates a child of root", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      expect(koso.toJSON()).toEqual({
        root: {
          id: "root",
          num: "0",
          name: "Root",
          children: [id1.name],
          assignee: null,
          reporter: null,
          status: null,
          statusTime: null,
        },
        [id1.name]: {
          id: id1.name,
          num: "1",
          name: "Task 1",
          children: [],
          assignee: null,
          reporter: "t@koso.app",
          status: null,
          statusTime: null,
        },
      });
    });
  });

  describe("linkNode", () => {
    it("link node 2 to node 1 succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
      ]);
      koso.linkNode(Node.parse("2"), Node.parse("1"), 0);

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1", "2"] },
        ["1"]: { id: "1", children: ["2"] },
        ["2"]: { id: "2", children: [] },
      });
    });

    it("link node 1 to child of node 1 throws (prevent cycle)", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1" },
      ]);
      expect(() =>
        koso.linkNode(Node.parse("1"), Node.parse("1"), 0),
      ).toThrow();
    });

    it("link node 1 to grandchild of node 1 throws (prevent cycle)", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);
      expect(() =>
        koso.linkNode(Node.parse("1"), Node.parse("2"), 0),
      ).toThrow();
    });
  });

  describe("moveNode", () => {
    it("move node 3 to child of node 1 as a peer of node 2 succeeds (reparent)", () => {
      init([
        { id: "root", name: "Root", children: ["1", "3"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
        { id: "3", name: "Task 3" },
      ]);

      koso.moveNode(Node.parse("3"), Node.parse("1"), 1);

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: ["2", "3"] },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    it("move node 3 to immediate child of node 1 succeeds (reparent)", () => {
      init([
        { id: "root", name: "Root", children: ["1", "3"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
        { id: "3", name: "Task 3" },
      ]);

      koso.moveNode(Node.parse("3"), Node.parse("1"), 0);

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: ["3", "2"] },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    it("move node 4 to be a child of node 3 succeeds (reparent)", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2", "3", "4"] },
        { id: "2", name: "Task 2" },
        { id: "3", name: "Task 3" },
        { id: "4", name: "Task 4" },
      ]);

      koso.moveNode(Node.parse("1/4"), Node.parse("1/3"), 0);

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: ["2", "3"] },
        ["2"]: { children: [] },
        ["3"]: { children: ["4"] },
        ["4"]: { children: [] },
      });
    });

    it("move node 2 to peer of itself throws (prevent duplicate)", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);

      expect(() =>
        koso.moveNode(Node.parse("2"), Node.parse("1"), 1),
      ).toThrow();
    });

    it("move node 4 to be the peer of node 2 succeeds (reorder)", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2", "3", "4"] },
        { id: "2", name: "Task 2" },
        { id: "3", name: "Task 3" },
        { id: "4", name: "Task 4" },
      ]);

      koso.moveNode(Node.parse("1/4"), Node.parse("1"), 1);

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: ["2", "4", "3"] },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
        ["4"]: { children: [] },
      });
    });

    it("move node 3 to be the peer of node 4 succeeds (reorder)", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2", "3", "4"] },
        { id: "2", name: "Task 2" },
        { id: "3", name: "Task 3" },
        { id: "4", name: "Task 4" },
      ]);

      koso.moveNode(Node.parse("1/3"), Node.parse("1"), 3);

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: ["2", "4", "3"] },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
        ["4"]: { children: [] },
      });
    });
  });

  describe("deleteNode", () => {
    it("delete node 2 from node 1 succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);

      koso.deleteNode(Node.parse("1/2"));

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1", "2"] },
        ["1"]: { id: "1", children: [] },
        ["2"]: { id: "2", children: [] },
      });
    });

    it("delete node 2 from node 1 succeeds and unlinks node 2 and deletes nodes 2, 4 and 6", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2", "7"] },
        { id: "1", name: "Task 1", children: [] },
        { id: "2", name: "Task 2", children: ["3", "4"] },
        { id: "3", name: "Task 3", children: ["5"] },
        { id: "4", name: "Task 4", children: ["6"] },
        { id: "5", name: "Task 5", children: [] },
        { id: "6", name: "Task 6", children: [] },
        { id: "7", name: "Task 7", children: ["3"] },
      ]);

      koso.deleteNode(Node.parse("2"));

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1", "7"] },
        ["1"]: { id: "1", children: [] },
        ["3"]: { id: "3", children: ["5"] },
        ["5"]: { id: "5", children: [] },
        ["7"]: { id: "7", children: ["3"] },
      });
    });

    it("delete node 2 from root succeeds and unlinks node 2", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2", children: ["3"] },
        { id: "3", name: "Task 3" },
      ]);

      koso.deleteNode(Node.parse("2"));

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { id: "1", children: ["2"] },
        ["2"]: { id: "2", children: ["3"] },
        ["3"]: { id: "3", children: [] },
      });
    });

    it("delete node 2 succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);

      koso.deleteNode(Node.parse("1/2"));

      expect(koso.toJSON()).toHaveProperty("root");
      expect(koso.toJSON()).toHaveProperty("1");
      expect(koso.toJSON()).not.toHaveProperty("2");
    });
  });

  describe("getTask", () => {
    it("retrieves task 1", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        {
          id: "1",
          num: "num",
          name: "Task 1",
          children: ["2"],
          reporter: "r@koso.app",
          assignee: "a@koso.app",
          status: "In Progress",
          statusTime: 123,
        },
      ]);

      expect(koso.getTask("1").toJSON()).toStrictEqual({
        id: "1",
        num: "num",
        name: "Task 1",
        children: ["2"],
        reporter: "r@koso.app",
        assignee: "a@koso.app",
        status: "In Progress",
        statusTime: 123,
      });
    });

    it("invalid task id throws an exception", () => {
      init([{ id: "root", name: "Root", children: ["1"] }]);
      expect(() => koso.getTask("non-existant-task")).toThrow();
    });
  });

  describe("getTasks", () => {
    it("fetches all tasks", () => {
      init([{ id: "root", name: "Root", children: ["1", "2", "3", "4"] }]);

      expect(koso.getTasks().map((task) => task.toJSON())).toMatchObject([
        { id: "root" },
        { id: "1" },
        { id: "2" },
        { id: "3" },
        { id: "4" },
      ]);
    });
  });

  describe("getParents", () => {
    beforeEach(() => {
      init([
        { id: "root", name: "Root", children: ["1", "3"] },
        { id: "1", name: "Task 1", children: ["2", "3"] },
        { id: "2", name: "Task 2", children: ["4"] },
        { id: "3", name: "Task 3", children: ["4"] },
      ]);
    });

    it("parents of 1 is root", () => {
      expect(koso.getParents("1")).toEqual(["root"]);
    });

    it("parents of 2 is 1", () => {
      expect(koso.getParents("2")).toEqual(["1"]);
    });

    it("parents of 3 are 1 and root", () => {
      expect(koso.getParents("3")).toEqual(["root", "1"]);
    });

    it("parents of 4 are 2 and 3", () => {
      expect(koso.getParents("4")).toEqual(["2", "3"]);
    });

    it("parents of root throws", () => {
      expect(() => koso.getParents("root")).toThrow();
    });
  });

  describe("getChildren", () => {
    beforeEach(() => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2", children: [] },
      ]);
    });

    it("retrieves task 1's children", () => {
      expect(koso.getChildren("1").toJSON()).toStrictEqual(["2"]);
    });

    it("retrieves empty list of children for leaf task", () => {
      expect(koso.getChildren("2").toJSON()).toStrictEqual([]);
    });

    it("invalid task id throws an exception", () => {
      expect(() => koso.getChildren("3")).toThrow();
    });
  });

  describe("getChildCount", () => {
    beforeEach(() => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: [] },
        { id: "2", name: "Task 2", children: [] },
      ]);
    });

    it("returns correct child count for a node with children", () => {
      expect(koso.getChildCount("root")).toBe(2);
    });

    it("returns zero for a node with no children", () => {
      expect(koso.getChildCount("1")).toBe(0);
    });

    it("throws an exception for an invalid task id", () => {
      expect(() => koso.getChildCount("non-existant-task")).toThrow();
    });
  });

  describe("hasChild", () => {
    beforeEach(() => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: [] },
        { id: "2", name: "Task 2", children: [] },
      ]);
    });

    it("returns true if task has the given child", () => {
      expect(koso.hasChild("root", "1")).toBe(true);
    });

    it("returns false if task does not have the given child", () => {
      expect(koso.hasChild("1", "2")).toBe(false);
    });

    it("throws an exception for an invalid task id", () => {
      expect(() => koso.hasChild("non-existant-task", "1")).toThrow();
    });
  });

  describe("getStatus", () => {
    it("returns Not Started for a task with no children and status Not Started", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2", "3"] },
        { id: "1", status: "Not Started" },
        { id: "2", status: "In Progress" },
        { id: "3", status: "Done" },
      ]);
      expect(koso.getStatus("1")).toBe("Not Started");
    });

    it("returns In Progress for a task with no children and status In Progress", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2", "3"] },
        { id: "1", status: "Not Started" },
        { id: "2", status: "In Progress" },
        { id: "3", status: "Done" },
      ]);
      expect(koso.getStatus("2")).toBe("In Progress");
    });

    it("returns Done for a task with no children and status Done", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2", "3"] },
        { id: "1", status: "Not Started" },
        { id: "2", status: "In Progress" },
        { id: "3", status: "Done" },
      ]);
      expect(koso.getStatus("3")).toBe("Done");
    });

    it("returns In Progress for a task with children in various states", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "3"] },
        { id: "2", status: "In Progress" },
        { id: "3", status: "Done" },
      ]);
      expect(koso.getStatus("1")).toBe("In Progress");
    });

    it("returns Done for a task with all children done", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "3"] },
        { id: "2", status: "Done" },
        { id: "3", status: "Done" },
      ]);
      expect(koso.getStatus("1")).toBe("Done");
    });

    it("returns Not Started for a task with all children not started", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "3"] },
        { id: "2", status: "Not Started" },
        { id: "3", status: "Not Started" },
      ]);
      expect(koso.getStatus("1")).toBe("Not Started");
    });
  });

  describe("getProgress", () => {
    const now = Date.now();

    it("parent node with all children done has 2 of 2 progress", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "3"] },
        { id: "2", status: "Done", statusTime: now },
        { id: "3", status: "Done", statusTime: now },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 0,
        done: 2,
        total: 2,
        lastStatusTime: now,
      });
    });

    it("parent node with one child in progress and one done has 1 of 2 progress", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "3"] },
        { id: "2", status: "In Progress", statusTime: now },
        { id: "3", status: "Done", statusTime: now },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 1,
        done: 1,
        total: 2,
        lastStatusTime: now,
      });
    });

    it("parent node with all children not started has 0 of 2 progress", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "3"] },
        { id: "2", status: "Not Started", statusTime: now },
        { id: "3", status: "Not Started", statusTime: now },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 0,
        done: 0,
        total: 2,
        lastStatusTime: now,
      });
    });

    it("parent node with mixed children statuses has correct progress", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "3", "4"] },
        { id: "2", status: "Not Started", statusTime: now },
        { id: "3", status: "In Progress", statusTime: now },
        { id: "4", status: "Done", statusTime: now },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 1,
        done: 1,
        total: 3,
        lastStatusTime: now,
      });
    });

    it("parent node with nested children has correct progress", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2"] },
        { id: "2", children: ["3"] },
        { id: "3", status: "Done", statusTime: now },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 0,
        done: 1,
        total: 1,
        lastStatusTime: now,
      });
    });

    it("parent node with multiple levels of nested children has correct progress", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2"] },
        { id: "2", children: ["3", "4"] },
        { id: "3", status: "In Progress", statusTime: now },
        { id: "4", status: "Done", statusTime: now },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 1,
        done: 1,
        total: 2,
        lastStatusTime: now,
      });
    });
  });

  describe("toNodes", () => {
    beforeEach(() => {
      koso.expanded = Set([]);
    });

    it("empty doc has no nodes", () => {
      expect(koso.nodes).toStrictEqual(List([root]));
    });

    it("doc with one task has one node", () => {
      init([{ id: "root", name: "Root", children: ["1"] }]);
      expect(koso.nodes).toStrictEqual(List([root, Node.parse("1")]));
    });

    it("doc with non-visible tasks returns root", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", status: "Done" },
      ]);
      expect(koso.nodes).toStrictEqual(List([root]));
    });

    it("doc with two tasks has two nodes", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
      ]);
      expect(koso.nodes).toStrictEqual(
        List([root, Node.parse("1"), Node.parse("2")]),
      );
    });

    it("doc with two tasks and one subtask has two nodes", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: ["3"] },
        { id: "2", name: "Task 2" },
        { id: "3", name: "Task 3" },
      ]);
      expect(koso.nodes).toStrictEqual(
        List([root, Node.parse("1"), Node.parse("2")]),
      );
    });

    it("doc with two tasks, one subtask, and parent is expanded has three nodes", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: ["3"] },
        { id: "2", name: "Task 2" },
        { id: "3", name: "Task 3" },
      ]);
      koso.expanded = Set([Node.parse("1")]);
      expect(koso.nodes).toStrictEqual(
        List([root, Node.parse("1"), Node.parse("1/3"), Node.parse("2")]),
      );
    });

    it("doc with two tasks, one linked subtask, and parent is expanded has three nodes", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);
      koso.expanded = Set([Node.parse("1")]);
      expect(koso.nodes).toStrictEqual(
        List([root, Node.parse("1"), Node.parse("1/2"), Node.parse("2")]),
      );
    });
  });

  describe("toJSON", () => {
    it("empty graph renders successfully", () => {
      expect(koso.toJSON()).toMatchObject({
        root: {
          num: "0",
          name: "Root",
          children: [],
          reporter: null,
          assignee: null,
          status: null,
        },
      });
    });

    it("graph with one task renders to json successfully", () => {
      init([
        { id: "root", num: "0", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", reporter: "t@koso.app" },
      ]);

      expect(koso.toJSON()).toMatchObject({
        root: { id: "root", num: "0", name: "Root", children: ["1"] },
        ["1"]: {
          id: "1",
          num: "1",
          name: "Task 1",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
          status: null,
        },
      });
    });

    it("populated graph renders to json successfully", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);

      expect(koso.toJSON()).toMatchObject({
        root: { id: "root", name: "Root", children: ["1", "2"] },
        ["1"]: { id: "1", name: "Task 1", children: ["2"] },
        ["2"]: { id: "2", name: "Task 2", children: [] },
      });
    });
  });

  describe("setTaskName", () => {
    it("set node 2's name succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
      ]);

      koso.setTaskName("2", "Edited Task 2");

      expect(koso.toJSON()).toMatchObject({
        root: { name: "Root" },
        ["1"]: { name: "Task 1" },
        ["2"]: { name: "Edited Task 2" },
      });
    });
  });

  describe("setAssignee", () => {
    it("set node 2's assignee succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
      ]);

      koso.setAssignee("2", USER);

      expect(koso.toJSON()).toMatchObject({
        root: { assignee: null },
        ["1"]: { assignee: null },
        ["2"]: { assignee: "t@koso.app" },
      });
    });
  });

  describe("setReporter", () => {
    it("set node 2's reporter succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", reporter: "t@koso.app" },
        { id: "2", name: "Task 2" },
      ]);

      koso.setReporter("2", {
        email: "new@koso.app",
        name: "New Test User",
        picture: "",
        exp: 0,
      });

      expect(koso.toJSON()).toMatchObject({
        root: { reporter: null },
        ["1"]: { reporter: "t@koso.app" },
        ["2"]: { reporter: "new@koso.app" },
      });
    });
  });

  describe("setTaskStatus", () => {
    it("set node 2's status to done succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
      ]);

      koso.setTaskStatus(Node.parse("2"), "Done", USER);

      expect(koso.toJSON()).toMatchObject({
        root: { status: null, children: ["1", "2"], assignee: null },
        ["1"]: { status: null, children: [], assignee: null },
        ["2"]: { status: "Done", children: [], assignee: null },
      });
    });

    it("set non-trailing node status to done succeeds and moves done task to end", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
      ]);

      koso.setTaskStatus(Node.parse("1"), "Done", USER);

      expect(koso.toJSON()).toMatchObject({
        root: { status: null, children: ["2", "1"], assignee: null },
        ["1"]: { status: "Done", children: [], assignee: null },
        ["2"]: { status: null, children: [], assignee: null },
      });
    });

    it("set node status to in-progress succeeds and moves done task to front and assigns to user", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
      ]);

      koso.setTaskStatus(Node.parse("2"), "In Progress", USER);

      expect(koso.toJSON()).toMatchObject({
        root: {
          status: null,
          children: ["2", "1"],
          assignee: null,
        },
        ["1"]: { status: null, children: [], assignee: null },
        ["2"]: {
          status: "In Progress",
          children: [],
          assignee: USER.email,
        },
      });
    });

    it("set node status to in-progress succeeds and moves done task to front and assigns to user", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
      ]);

      koso.setTaskStatus(Node.parse("2"), "In Progress", OTHER_USER);

      expect(koso.toJSON()).toMatchObject({
        root: {
          status: null,
          children: ["2", "1"],
          assignee: null,
        },
        ["1"]: { status: null, children: [], assignee: null },
        ["2"]: {
          status: "In Progress",
          children: [],
          assignee: OTHER_USER.email,
        },
      });
    });

    it("setting a task to Done moves task to the bottom", () => {
      init([
        { id: "root", name: "Root", children: ["t1", "t2", "t3", "t4", "t5"] },
      ]);

      koso.setTaskStatus(Node.parse("t2"), "Done", USER);
      const children = koso.toJSON().root.children;

      expect(children).toEqual(["t1", "t3", "t4", "t5", "t2"]);
    });

    it("setting a task to In Progress moves task to the top", () => {
      init([
        { id: "root", name: "Root", children: ["t1", "t2", "t3", "t4", "t5"] },
      ]);

      koso.setTaskStatus(Node.parse("t4"), "In Progress", USER);
      const children = koso.toJSON().root.children;

      expect(children).toEqual(["t4", "t1", "t2", "t3", "t5"]);
    });

    it("setting task to In Progress moves next to the last In Progress task", () => {
      init([
        { id: "root", name: "Root", children: ["t1", "t2", "t3", "t4", "t5"] },
        { id: "t2", status: "In Progress" },
        { id: "t5", status: "Done" },
      ]);

      koso.setTaskStatus(Node.parse("t4"), "In Progress", USER);
      const children = koso.toJSON().root.children;

      expect(children).toEqual(["t1", "t2", "t4", "t3", "t5"]);
    });

    it("setting task to Done moves next to the first Done task", () => {
      init([
        { id: "root", name: "Root", children: ["t1", "t2", "t3", "t4", "t5"] },
        { id: "t1", status: "In Progress" },
        { id: "t4", status: "Done" },
      ]);

      koso.setTaskStatus(Node.parse("t2"), "Done", USER);
      const children = koso.toJSON().root.children;

      expect(children).toEqual(["t1", "t3", "t2", "t4", "t5"]);
    });

    it("setting a Done task to In Progress moves up", () => {
      init([
        { id: "root", name: "Root", children: ["t1", "t2", "t3", "t4", "t5"] },
        { id: "t1", status: "In Progress" },
        { id: "t4", status: "Done" },
        { id: "t5", status: "Done" },
      ]);

      koso.setTaskStatus(Node.parse("t5"), "In Progress", USER);
      const children = koso.toJSON().root.children;

      expect(children).toEqual(["t1", "t5", "t2", "t3", "t4"]);
    });
  });
});
