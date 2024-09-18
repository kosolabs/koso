import { List, Set } from "immutable";
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

describe("Koso tests", () => {
  const root = new Node();
  let koso: Koso;
  beforeEach(() => {
    koso = new Koso("project-id", new Y.Doc());
    koso.handleClientMessage(() => {});
    koso.handleServerMessage(EMPTY_SYNC_RESPONSE);
  });

  describe("getTask", () => {
    it("retrieves task 2", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(id1, 0, USER, "Task 2");
      expect(koso.getTask(id2.name)).toStrictEqual({
        id: id2.name,
        num: "2",
        name: "Task 2",
        children: [],
        reporter: "t@koso.app",
        assignee: null,
        status: null,
      });
    });

    it("invalid task id throws an exception", () => {
      koso.insertNode(root, 0, USER, "Task 1");
      koso.insertNode(root, 1, USER, "Task 2");
      expect(() => koso.getTask("non-existant-task")).toThrow();
    });
  });

  describe("getChildren", () => {
    it("retrieves task 1's children", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(id1, 0, USER, "Task 2");
      expect(koso.getChildren(id1.name)).toStrictEqual([id2.name]);
    });

    it("retrieves empty list of children for leaf task", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(id1, 0, USER, "Task 2");
      expect(koso.getChildren(id2.name)).toStrictEqual([]);
    });

    it("invalid task id throws an exception", () => {
      koso.insertNode(root, 0, USER, "Task 1");
      koso.insertNode(root, 1, USER, "Task 2");
      expect(() => koso.getChildren("id3")).toThrow();
    });
  });

  describe("toNodes", () => {
    it("empty doc has no nodes", () => {
      expect(get(koso.nodes)).toStrictEqual(List([root]));
    });

    it("doc with one task has one node", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      expect(get(koso.nodes)).toStrictEqual(List([root, id1]));
    });

    it("doc with two tasks has two nodes", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(root, 1, USER, "Task 2");
      expect(get(koso.nodes)).toStrictEqual(List([root, id1, id2]));
    });

    it("doc with two tasks and one subtask has two nodes", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(root, 1, USER, "Task 2");
      koso.insertNode(id1, 0, USER, "Task 3");
      expect(get(koso.nodes)).toStrictEqual(List([root, id1, id2]));
    });

    it("doc with two tasks, one subtask, and parent is expanded has three nodes", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(root, 1, USER, "Task 2");
      const id3 = koso.insertNode(id1, 0, USER, "Task 3");
      koso.expanded.set(Set([id1]));
      expect(get(koso.nodes)).toStrictEqual(List([root, id1, id3, id2]));
    });

    it("doc with two tasks, one linked subtask, and parent is expanded has three nodes", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(root, 1, USER, "Task 2");
      koso.linkNode(id2, id1.name, 0);
      koso.expanded.set(Set([id1]));
      const lid = id1.child(id2.name);
      expect(get(koso.nodes)).toStrictEqual(List([root, id1, lid, id2]));
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
      const id1 = koso.insertNode(root, 0, USER, "Task 1");

      expect(koso.toJSON()).toMatchObject({
        root: { id: "root", num: "0", name: "Root", children: [id1.name] },
        [id1.name]: {
          id: id1.name,
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
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(id1, 0, USER, "Task 2");

      expect(koso.toJSON()).toMatchObject({
        root: { id: "root", num: "0", name: "Root", children: [id1.name] },
        [id1.name]: {
          id: id1.name,
          num: "1",
          name: "Task 1",
          children: [id2.name],
        },
        [id2.name]: { id: id2.name, num: "2", name: "Task 2", children: [] },
      });
    });

    it("link node 2 to node 1 succeeds", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(root, 1, USER, "Task 2");
      koso.linkNode(id2, id1.name, 0);

      expect(koso.toJSON()).toMatchObject({
        root: { children: [id1.name, id2.name] },
        [id1.name]: { id: id1.name, children: [id2.name] },
        [id2.name]: { id: id2.name, children: [] },
      });
    });

    it("delete node 2 from node 1 succeeds", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(root, 1, USER, "Task 2");
      koso.linkNode(id2, id1.name, 0);
      koso.expanded.set(Set([id1]));

      koso.deleteNode(id1.child(id2.name));

      expect(koso.toJSON()).toMatchObject({
        root: { children: [id1.name, id2.name] },
        [id1.name]: { id: id1.name, children: [] },
        [id2.name]: { id: id2.name, children: [] },
      });
    });

    it("delete node 2 from node 1 succeeds and unlinks node 2 and deletes nodes 2, 4 and 6", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(root, 1, USER, "Task 2");
      const id3 = koso.insertNode(id2, 0, USER, "Task 3");
      const id4 = koso.insertNode(id2, 1, USER, "Task 4");
      koso.expanded.set(Set([id1, id2, id3, id4]));
      const id5 = koso.insertNode(id3, 0, USER, "Task 5");
      koso.insertNode(id4, 0, USER, "Task 6");
      const id7 = koso.insertNode(root, 2, USER, "Task 7");
      koso.linkNode(id3, id7.name, 0);
      koso.deleteNode(id2);

      expect(koso.toJSON()).toMatchObject({
        root: { children: [id1.name, id7.name] },
        [id1.name]: { id: id1.name, children: [] },
        [id7.name]: { id: id7.name, children: [id3.name] },
        [id3.name]: { id: id3.name, children: [id5.name] },
        [id5.name]: { id: id5.name, children: [] },
      });
    });

    it("delete node 2 from root succeeds and unlinks node 2", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(root, 1, USER, "Task 2");
      const id3 = koso.insertNode(id2, 0, USER, "Task 3");
      koso.linkNode(id2, id1.name, 0);
      koso.deleteNode(id2);

      expect(koso.toJSON()).toMatchObject({
        root: { children: [id1.name] },
        [id1.name]: { id: id1.name, children: [id2.name] },
        [id2.name]: { id: id2.name, children: [id3.name] },
        [id3.name]: { id: id3.name, children: [] },
      });
    });

    it("delete node 2 succeeds", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(id1, 0, USER, "Task 2");
      koso.expanded.set(Set([id1]));

      koso.deleteNode(id2);

      expect(koso.toJSON()).toHaveProperty("root");
      expect(koso.toJSON()).toHaveProperty(id1.name);
      expect(koso.toJSON()).not.toHaveProperty(id2.name);
    });

    it("link node 1 to child of node 1 throws (prevent cycle)", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      expect(() => koso.linkNode(id1, id1.name, 0)).toThrow();
    });

    it("link node 1 to grandchild of node 1 throws (prevent cycle)", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(id1, 0, USER, "Task 1");
      expect(() => koso.linkNode(id1, id2.name, 0)).toThrow();
    });

    it("move node 3 to child of node 1 as a peer of node 2 succeeds (reparent)", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(id1, 0, USER, "Task 2");
      const id3 = koso.insertNode(root, 1, USER, "Task 3");
      koso.expanded.set(Set([id1]));

      koso.moveNode(id3, id1.name, 1);

      expect(koso.toJSON()).toMatchObject({
        root: { children: [id1.name] },
        [id1.name]: { children: [id2.name, id3.name] },
        [id2.name]: { children: [] },
        [id3.name]: { children: [] },
      });
    });

    it("move node 3 to immediate child of node 1 succeeds (reparent)", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(id1, 0, USER, "Task 2");
      const id3 = koso.insertNode(root, 1, USER, "Task 3");
      koso.expanded.set(Set([id1]));

      koso.moveNode(id3, id1.name, 0);

      expect(koso.toJSON()).toMatchObject({
        root: { children: [id1.name] },
        [id1.name]: { children: [id3.name, id2.name] },
        [id2.name]: { children: [] },
        [id3.name]: { children: [] },
      });
    });

    it("move node 4 to be a child of node 3 succeeds (reparent)", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(id1, 0, USER, "Task 2");
      const id3 = koso.insertNode(id1, 1, USER, "Task 3");
      const id4 = koso.insertNode(id1, 2, USER, "Task 4");
      koso.expanded.set(Set([id1, id3]));

      koso.moveNode(id4, id3.name, 0);

      expect(koso.toJSON()).toMatchObject({
        root: { children: [id1.name] },
        [id1.name]: { children: [id2.name, id3.name] },
        [id2.name]: { children: [] },
        [id3.name]: { children: [id4.name] },
        [id4.name]: { children: [] },
      });
    });

    it("move node 2 to peer of itself throws (prevent duplicate)", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(root, 0, USER, "Task 2");
      koso.linkNode(id2, id1.name, 0);
      koso.expanded.set(Set([id1.child(id2.name)]));

      expect(() => koso.moveNode(id2, id1.name, 1)).toThrow();
    });

    it("move node 4 to be the peer of node 2 succeeds (reorder)", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(id1, 0, USER, "Task 2");
      const id3 = koso.insertNode(id1, 1, USER, "Task 3");
      const id4 = koso.insertNode(id1, 2, USER, "Task 4");
      koso.expanded.set(Set([id1]));

      koso.moveNode(id4, id1.name, 1);

      expect(koso.toJSON()).toMatchObject({
        root: { children: [id1.name] },
        [id1.name]: { children: [id2.name, id4.name, id3.name] },
        [id2.name]: { children: [] },
        [id3.name]: { children: [] },
        [id4.name]: { children: [] },
      });
    });

    it("move node 3 to be the peer of node 4 succeeds (reorder)", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(id1, 0, USER, "Task 2");
      const id3 = koso.insertNode(id1, 1, USER, "Task 3");
      const id4 = koso.insertNode(id1, 2, USER, "Task 4");
      koso.expanded.set(Set([id1]));

      koso.moveNode(id3, id1.name, 3);

      expect(koso.toJSON()).toMatchObject({
        root: { children: [id1.name] },
        [id1.name]: { children: [id2.name, id4.name, id3.name] },
        [id2.name]: { children: [] },
        [id3.name]: { children: [] },
        [id4.name]: { children: [] },
      });
    });
  });

  describe("setTaskName", () => {
    it("set node 2's name succeeds", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(root, 0, USER, "Task 2");

      koso.setTaskName(id2.name, "Edited Task 2");

      expect(koso.toJSON()).toMatchObject({
        root: { name: "Root" },
        [id1.name]: { name: "Task 1" },
        [id2.name]: { name: "Edited Task 2" },
      });
    });
  });

  describe("setAssignee", () => {
    it("set node 2's assignee succeeds", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(root, 0, USER, "Task 2");

      koso.setAssignee(id2.name, USER);

      expect(koso.toJSON()).toMatchObject({
        root: { assignee: null },
        [id1.name]: { assignee: null },
        [id2.name]: { assignee: "t@koso.app" },
      });
    });
  });

  describe("setReporter", () => {
    it("set node 2's reporter succeeds", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(root, 0, USER, "Task 2");

      koso.setReporter(id2.name, {
        email: "new@koso.app",
        name: "New Test User",
        picture: "",
        exp: 0,
      });

      expect(koso.toJSON()).toMatchObject({
        root: { reporter: null },
        [id1.name]: { reporter: "t@koso.app" },
        [id2.name]: { reporter: "new@koso.app" },
      });
    });
  });

  describe("setTaskStatus", () => {
    it("set node 2's reporter succeeds", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(root, 0, USER, "Task 2");

      koso.setTaskStatus(id2, "Done");

      expect(koso.toJSON()).toMatchObject({
        root: { status: null },
        [id1.name]: { status: null },
        [id2.name]: { status: "Done" },
      });
    });
  });

  describe("getProgress", () => {
    it("leaf node that is not started has 0 of 1 progress", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");

      expect(koso.getProgress(id1.name)).toEqual({
        numer: 0,
        denom: 1,
      });
    });

    it("leaf node that is in progress has 0 of 1 progress", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      koso.setTaskStatus(id1, "In Progress");

      expect(koso.getProgress(id1.name)).toEqual({
        numer: 0,
        denom: 1,
      });
    });

    it("leaf node that is done has 1 of 1 progress", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      koso.setTaskStatus(id1, "Done");

      expect(koso.getProgress(id1.name)).toEqual({
        numer: 1,
        denom: 1,
      });
    });

    it("parent node with 2 not started children has 0 of 2 progress", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      koso.insertNode(id1, 0, USER, "Task 2");
      koso.insertNode(id1, 0, USER, "Task 3");

      expect(koso.getProgress(id1.name)).toEqual({
        numer: 0,
        denom: 2,
      });
    });

    it("parent node with 2 in progress children has 0 of 2 progress", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(id1, 0, USER, "Task 2");
      const id3 = koso.insertNode(id1, 0, USER, "Task 3");
      koso.setTaskStatus(id2, "In Progress");
      koso.setTaskStatus(id3, "In Progress");

      expect(koso.getProgress(id1.name)).toEqual({
        numer: 0,
        denom: 2,
      });
    });

    it("parent node with 1 done and 1 in progress children has 1 of 2 progress", () => {
      const id1 = koso.insertNode(root, 0, USER, "Task 1");
      const id2 = koso.insertNode(id1, 0, USER, "Task 2");
      const id3 = koso.insertNode(id1, 0, USER, "Task 3");
      koso.setTaskStatus(id2, "Done");
      koso.setTaskStatus(id3, "In Progress");

      expect(koso.getProgress(id1.name)).toEqual({
        numer: 1,
        denom: 2,
      });
    });
  });
});
