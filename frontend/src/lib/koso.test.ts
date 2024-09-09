import * as encoding from "lib0/encoding";
import { get } from "svelte/store";
import { beforeEach, describe, expect, it } from "vitest";
import * as Y from "yjs";
import type { User } from "./auth";
import { Koso, Node } from "./koso";

const USER: User = {
  email: "t@koso.app",
  name: "Test User",
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

function task(nodeId: string): string {
  return nodeId.split(Node.separator).slice(-1)[0];
}

describe("Koso tests", () => {
  let koso: Koso;
  beforeEach(() => {
    koso = new Koso("project-id", new Y.Doc());
    koso.handleClientMessage(() => {});
    koso.handleServerMessage(EMPTY_SYNC_RESPONSE);
  });

  describe("getTask", () => {
    it("retrieves task 2", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode(id1), 0, "Task 2", USER);
      expect(koso.getTask(task(id2))).toStrictEqual({
        id: task(id2),
        num: "2",
        name: "Task 2",
        children: [],
        reporter: "t@koso.app",
        assignee: null,
        status: null,
      });
    });

    it("invalid task id throws an exception", () => {
      koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      koso.insertNode(koso.getNode("root"), 1, "Task 2", USER);
      expect(() => koso.getTask("non-existant-task")).toThrow();
    });
  });

  describe("getChildren", () => {
    it("retrieves task 1's children", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode(id1), 0, "Task 2", USER);
      expect(koso.getChildren(task(id1))).toStrictEqual([task(id2)]);
    });

    it("retrieves empty list of children for leaf task", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode(id1), 0, "Task 2", USER);
      expect(koso.getChildren(task(id2))).toStrictEqual([]);
    });

    it("invalid task id throws an exception", () => {
      koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      koso.insertNode(koso.getNode("root"), 1, "Task 2", USER);
      expect(() => koso.getChildren("id3")).toThrow();
    });
  });

  describe("toNodes", () => {
    function root(): Node {
      return new Node(koso, [], 0, 0);
    }

    function node(id: string, offset: number, index: number): Node {
      return new Node(koso, id.split(Node.separator), offset, index);
    }

    it("empty doc has no nodes", () => {
      expect(get(koso.nodes)).toStrictEqual(new Map([["root", root()]]));
    });

    it("doc with one task has one node", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      expect(get(koso.nodes)).toStrictEqual(
        new Map([
          ["root", root()],
          [id1, node(id1, 0, 1)],
        ]),
      );
    });

    it("doc with two tasks has two nodes", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode("root"), 1, "Task 2", USER);
      expect(get(koso.nodes)).toStrictEqual(
        new Map([
          ["root", root()],
          [id1, node(id1, 0, 1)],
          [id2, node(id2, 1, 2)],
        ]),
      );
    });

    it("doc with two tasks and one subtask has two nodes", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode("root"), 1, "Task 2", USER);
      koso.insertNode(koso.getNode(id1), 0, "Task 3", USER);
      expect(get(koso.nodes)).toStrictEqual(
        new Map([
          ["root", root()],
          [id1, node(id1, 0, 1)],
          [id2, node(id2, 1, 2)],
        ]),
      );
    });

    it("doc with two tasks, one subtask, and parent is expanded has three nodes", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode("root"), 1, "Task 2", USER);
      const id3 = koso.insertNode(koso.getNode(id1), 0, "Task 3", USER);
      koso.expanded.set(new Set([id1]));
      expect(get(koso.nodes)).toStrictEqual(
        new Map([
          ["root", root()],
          [id1, node(id1, 0, 1)],
          [id3, node(id3, 0, 2)],
          [id2, node(id2, 1, 3)],
        ]),
      );
    });

    it("doc with two tasks, one linked subtask, and parent is expanded has three nodes", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode("root"), 1, "Task 2", USER);
      const node2 = koso.getNode(id2);
      const lid = [id1, id2].join(Node.separator);
      koso.linkNode(node2, task(id1), 0);
      koso.expanded.set(new Set([id1]));
      expect(get(koso.nodes)).toStrictEqual(
        new Map([
          ["root", root()],
          [id1, node(id1, 0, 1)],
          [lid, node(lid, 0, 2)],
          [id2, node(id2, 1, 3)],
        ]),
      );
    });
  });

  describe("toParents", () => {
    it("empty doc has no parents", () => {
      expect(get(koso.parents)).toStrictEqual({});
    });

    it("doc with one task has root parent", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      expect(get(koso.parents)).toStrictEqual({
        [id1]: ["root"],
      });
    });

    it("doc with two tasks has root as parent for both", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode("root"), 1, "Task 2", USER);
      expect(get(koso.parents)).toStrictEqual({
        [id1]: ["root"],
        [id2]: ["root"],
      });
    });

    it("doc with a task with two parents", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode("root"), 1, "Task 2", USER);
      koso.linkNode(koso.getNode(id2), id1, 0);

      expect(get(koso.parents)).toStrictEqual({
        [id1]: ["root"],
        [id2]: ["root", id1],
      });
    });
  });

  describe("graph", () => {
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

    it("graph with one root node renders to json successfully", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);

      expect(koso.toJSON()).toMatchObject({
        root: { id: "root", num: "0", name: "Root", children: [id1] },
        [id1]: {
          id: id1,
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
      const id1 = task(
        koso.insertNode(koso.getNode("root"), 0, "Task 1", USER),
      );
      const id2 = task(koso.insertNode(koso.getNode(id1), 0, "Task 2", USER));

      expect(koso.toJSON()).toMatchObject({
        root: { id: "root", num: "0", name: "Root", children: [id1] },
        [id1]: { id: id1, num: "1", name: "Task 1", children: [id2] },
        [id2]: { id: id2, num: "2", name: "Task 2", children: [] },
      });
    });

    it("link node 2 to node 1 succeeds", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode("root"), 1, "Task 2", USER);

      koso.linkNode(koso.getNode(id2), id1, 0);

      expect(koso.toJSON()).toMatchObject({
        root: { children: [id1, id2] },
        [id1]: { id: id1, children: [id2] },
        [id2]: { id: id2, children: [] },
      });
    });

    it("delete node 2 from node 1 succeeds", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode("root"), 1, "Task 2", USER);
      koso.linkNode(koso.getNode(id2), id1, 0);
      koso.expanded.set(new Set([id1]));

      koso.deleteNode(koso.getNode([id1, id2].join(Node.separator)));

      expect(koso.toJSON()).toMatchObject({
        root: { children: [id1, id2] },
        [id1]: { id: id1, children: [] },
        [id2]: { id: id2, children: [] },
      });
    });

    it("delete node 2 from node 1 succeeds and unlinks node 2 and deletes nodes 2, 4 and 6", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode("root"), 1, "Task 2", USER);
      const id3 = koso.insertNode(koso.getNode(id2), 0, "Task 3", USER);
      const id4 = koso.insertNode(koso.getNode(id2), 1, "Task 4", USER);
      koso.expanded.set(new Set([id1, id2, id3, id4]));
      const id5 = koso.insertNode(koso.getNode(id3), 0, "Task 5", USER);
      koso.insertNode(koso.getNode(id4), 0, "Task 6", USER);
      const id7 = koso.insertNode(koso.getNode("root"), 2, "Task 7", USER);
      koso.linkNode(koso.getNode(id3), task(id7), 0);
      koso.deleteNode(koso.getNode(id2));

      expect(koso.toJSON()).toMatchObject({
        root: { children: [task(id1), task(id7)] },
        [task(id1)]: { id: task(id1), children: [] },
        [task(id7)]: { id: task(id7), children: [task(id3)] },
        [task(id3)]: { id: task(id3), children: [task(id5)] },
        [task(id5)]: { id: task(id5), children: [] },
      });
    });

    it("delete node 2 from root succeeds and unlinks node 2", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode("root"), 1, "Task 2", USER);
      const id3 = koso.insertNode(koso.getNode(id2), 0, "Task 3", USER);
      koso.linkNode(koso.getNode(id2), task(id1), 0);
      koso.deleteNode(koso.getNode(id2));

      expect(koso.toJSON()).toMatchObject({
        root: { children: [task(id1)] },
        [task(id1)]: { id: task(id1), children: [task(id2)] },
        [task(id2)]: { id: task(id2), children: [task(id3)] },
        [task(id3)]: { id: task(id3), children: [] },
      });
    });

    it("delete node 2 succeeds", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode(id1), 0, "Task 2", USER);
      koso.expanded.set(new Set([id1]));

      koso.deleteNode(koso.getNode(id2));

      expect(koso.toJSON()).toHaveProperty("root");
      expect(koso.toJSON()).toHaveProperty(task(id1));
      expect(koso.toJSON()).not.toHaveProperty(task(id2));
    });

    it("link node 1 to child of node 1 throws (prevent cycle)", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      expect(() => koso.linkNode(koso.getNode(id1), task(id1), 0)).toThrow();
    });

    it("link node 1 to grandchild of node 1 throws (prevent cycle)", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode(id1), 0, "Task 1", USER);
      expect(() => koso.linkNode(koso.getNode(id1), task(id2), 0)).toThrow();
    });

    it("move node 3 to child of node 1 as a peer of node 2 succeeds (reparent)", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode(id1), 0, "Task 2", USER);
      const id3 = koso.insertNode(koso.getNode("root"), 1, "Task 3", USER);
      koso.expanded.set(new Set([id1]));

      koso.moveNode(koso.getNode(id3), id1, 1);

      expect(koso.toJSON()).toMatchObject({
        root: { children: [task(id1)] },
        [task(id1)]: { children: [task(id2), task(id3)] },
        [task(id2)]: { children: [] },
        [task(id3)]: { children: [] },
      });
    });

    it("move node 3 to immediate child of node 1 succeeds (reparent)", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode(id1), 0, "Task 2", USER);
      const id3 = koso.insertNode(koso.getNode("root"), 1, "Task 3", USER);
      koso.expanded.set(new Set([id1]));

      koso.moveNode(koso.getNode(id3), id1, 0);

      expect(koso.toJSON()).toMatchObject({
        root: { children: [task(id1)] },
        [task(id1)]: { children: [task(id3), task(id2)] },
        [task(id2)]: { children: [] },
        [task(id3)]: { children: [] },
      });
    });

    it("move node 4 to be a child of node 3 succeeds (reparent)", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode(id1), 0, "Task 2", USER);
      const id3 = koso.insertNode(koso.getNode(id1), 1, "Task 3", USER);
      const id4 = koso.insertNode(koso.getNode(id1), 2, "Task 4", USER);
      koso.expanded.set(new Set([id1, id3]));

      koso.moveNode(koso.getNode(id4), task(id3), 0);

      expect(koso.toJSON()).toMatchObject({
        root: { children: [task(id1)] },
        [task(id1)]: { children: [task(id2), task(id3)] },
        [task(id2)]: { children: [] },
        [task(id3)]: { children: [task(id4)] },
        [task(id4)]: { children: [] },
      });
    });

    it("move node 2 to peer of itself throws (prevent duplicate)", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode("root"), 0, "Task 2", USER);
      koso.linkNode(koso.getNode(id2), task(id1), 0);
      koso.expanded.set(new Set([Node.id([id1, id2])]));

      expect(() => koso.moveNode(koso.getNode(id2), task(id1), 1)).toThrow();
    });

    it("move node 4 to be the peer of node 2 succeeds (reorder)", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode(id1), 0, "Task 2", USER);
      const id3 = koso.insertNode(koso.getNode(id1), 1, "Task 3", USER);
      const id4 = koso.insertNode(koso.getNode(id1), 2, "Task 4", USER);
      koso.expanded.set(new Set([id1]));

      koso.moveNode(koso.getNode(id4), id1, 1);

      expect(koso.toJSON()).toMatchObject({
        root: { children: [task(id1)] },
        [task(id1)]: { children: [task(id2), task(id4), task(id3)] },
        [task(id2)]: { children: [] },
        [task(id3)]: { children: [] },
        [task(id4)]: { children: [] },
      });
    });

    it("move node 3 to be the peer of node 4 succeeds (reorder)", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode(id1), 0, "Task 2", USER);
      const id3 = koso.insertNode(koso.getNode(id1), 1, "Task 3", USER);
      const id4 = koso.insertNode(koso.getNode(id1), 2, "Task 4", USER);
      koso.expanded.set(new Set([id1]));

      koso.moveNode(koso.getNode(id3), id1, 3);

      expect(koso.toJSON()).toMatchObject({
        root: { children: [task(id1)] },
        [task(id1)]: { children: [task(id2), task(id4), task(id3)] },
        [task(id2)]: { children: [] },
        [task(id3)]: { children: [] },
        [task(id4)]: { children: [] },
      });
    });
  });

  describe("setTaskName", () => {
    it("set node 2's name succeeds", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode("root"), 0, "Task 2", USER);

      koso.setTaskName(id2, "Edited Task 2");

      expect(koso.toJSON()).toMatchObject({
        root: { name: "Root" },
        [id1]: { name: "Task 1" },
        [id2]: { name: "Edited Task 2" },
      });
    });
  });

  describe("setAssignee", () => {
    it("set node 2's assignee succeeds", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode("root"), 0, "Task 2", USER);

      koso.setAssignee(id2, USER);

      expect(koso.toJSON()).toMatchObject({
        root: { assignee: null },
        [id1]: { assignee: null },
        [id2]: { assignee: "t@koso.app" },
      });
    });
  });

  describe("setReporter", () => {
    it("set node 2's reporter succeeds", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode("root"), 0, "Task 2", USER);

      koso.setReporter(id2, {
        email: "new@koso.app",
        name: "New Test User",
        picture: "",
        exp: 0,
      });

      expect(koso.toJSON()).toMatchObject({
        root: { reporter: null },
        [id1]: { reporter: "t@koso.app" },
        [id2]: { reporter: "new@koso.app" },
      });
    });
  });

  describe("setTaskStatus", () => {
    it("set node 2's reporter succeeds", () => {
      const id1 = koso.insertNode(koso.getNode("root"), 0, "Task 1", USER);
      const id2 = koso.insertNode(koso.getNode("root"), 0, "Task 2", USER);

      koso.setTaskStatus(id2, "Done");

      expect(koso.toJSON()).toMatchObject({
        root: { status: null },
        [id1]: { status: null },
        [id2]: { status: "Done" },
      });
    });
  });
});
