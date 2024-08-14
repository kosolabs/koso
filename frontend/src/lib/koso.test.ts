import * as encoding from "lib0/encoding";
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
  let koso: Koso;
  beforeEach(() => {
    koso = new Koso("project-id", new Y.Doc());
    koso.handleClientMessage(() => {});
    koso.handleServerMessage(EMPTY_SYNC_RESPONSE);
  });

  describe("getTask", () => {
    it("retrieves task 2", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode(id1, 0, "Task 2", USER);
      expect(koso.getTask(id2)).toStrictEqual({
        id: id2,
        num: "2",
        name: "Task 2",
        children: [],
        reporter: "t@koso.app",
        assignee: null,
        status: null,
      });
    });

    it("invalid task id throws an exception", () => {
      koso.insertNode("root", 0, "Task 1", USER);
      koso.insertNode("root", 1, "Task 2", USER);
      expect(() => koso.getTask("id3")).toThrow();
    });
  });

  describe("getChildren", () => {
    it("retrieves task 1's children", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode(id1, 0, "Task 2", USER);
      expect(koso.getChildren(id1)).toStrictEqual([id2]);
    });

    it("retrieves empty list of children for leaf task", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode(id1, 0, "Task 2", USER);
      expect(koso.getChildren(id2)).toStrictEqual([]);
    });

    it("invalid task id throws an exception", () => {
      koso.insertNode("root", 0, "Task 1", USER);
      koso.insertNode("root", 1, "Task 2", USER);
      expect(() => koso.getChildren("id3")).toThrow();
    });
  });

  describe("getOffset", () => {
    it("offset of root is 0", () => {
      expect(koso.getOffset(new Node([]))).toBe(0);
    });

    it("offset of first node is 0", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      koso.insertNode("root", 1, "Task 2", USER);
      expect(koso.getOffset(new Node([id1]))).toBe(0);
    });

    it("offset of first node is 0", () => {
      koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode("root", 1, "Task 2", USER);
      expect(koso.getOffset(new Node([id2]))).toBe(1);
    });
  });

  describe("graph", () => {
    it("empty graph renders successfully", () => {
      expect(koso.yGraph.toJSON()).toMatchObject({
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
      const id1 = koso.insertNode("root", 0, "Task 1", USER);

      expect(koso.yGraph.toJSON()).toMatchObject({
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
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode(id1, 0, "Task 2", USER);

      expect(koso.yGraph.toJSON()).toMatchObject({
        root: { id: "root", num: "0", name: "Root", children: [id1] },
        [id1]: { id: id1, num: "1", name: "Task 1", children: [id2] },
        [id2]: { id: id2, num: "2", name: "Task 2", children: [] },
      });
    });

    it("link node 2 to node 1 succeeds", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode("root", 1, "Task 2", USER);

      koso.linkNode(id2, id1, 0);

      expect(koso.yGraph.toJSON()).toMatchObject({
        root: { children: [id1, id2] },
        [id1]: { id: id1, children: [id2] },
        [id2]: { id: id2, children: [] },
      });
    });

    it("unlink node 2 from node 1 succeeds", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode("root", 1, "Task 2", USER);
      koso.linkNode(id2, id1, 0);

      koso.unlinkNode(new Node([id1, id2]));

      expect(koso.yGraph.toJSON()).toMatchObject({
        root: { children: [id1, id2] },
        [id1]: { id: id1, children: [] },
        [id2]: { id: id2, children: [] },
      });
    });

    it("delete node 2 succeeds", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode(id1, 0, "Task 2", USER);

      koso.deleteNode(new Node([id1, id2]));

      expect(koso.yGraph.toJSON()).toHaveProperty("root");
      expect(koso.yGraph.toJSON()).toHaveProperty(id1);
      expect(koso.yGraph.toJSON()).not.toHaveProperty(id2);
    });

    it("move node 3 to child of node 1 as a peer of node 2 succeeds", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode(id1, 0, "Task 2", USER);
      const id3 = koso.insertNode("root", 1, "Task 3", USER);

      koso.moveNode(id3, "root", 1, id1, 1);

      expect(koso.yGraph.toJSON()).toMatchObject({
        root: { children: [id1] },
        [id1]: { children: [id2, id3] },
        [id2]: { children: [] },
        [id3]: { children: [] },
      });
    });

    it("move node 3 to immediate child of node 1 succeeds", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode(id1, 0, "Task 2", USER);
      const id3 = koso.insertNode("root", 1, "Task 3", USER);

      koso.moveNode(id3, "root", 1, id1, 0);

      expect(koso.yGraph.toJSON()).toMatchObject({
        root: { children: [id1] },
        [id1]: { children: [id3, id2] },
        [id2]: { children: [] },
        [id3]: { children: [] },
      });
    });

    it("move node 4 to be a child of node 3 succeeds", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode(id1, 0, "Task 2", USER);
      const id3 = koso.insertNode(id1, 1, "Task 3", USER);
      const id4 = koso.insertNode(id1, 2, "Task 4", USER);

      koso.moveNode(id4, id1, 2, id3, 0);

      expect(koso.yGraph.toJSON()).toMatchObject({
        root: { children: [id1] },
        [id1]: { children: [id2, id3] },
        [id2]: { children: [] },
        [id3]: { children: [id4] },
        [id4]: { children: [] },
      });
    });

    it("move node 4 to be the peer of node 2 succeeds", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode(id1, 0, "Task 2", USER);
      const id3 = koso.insertNode(id1, 1, "Task 3", USER);
      const id4 = koso.insertNode(id1, 2, "Task 4", USER);

      koso.moveNode(id4, id1, 2, id1, 1);

      expect(koso.yGraph.toJSON()).toMatchObject({
        root: { children: [id1] },
        [id1]: { children: [id2, id4, id3] },
        [id2]: { children: [] },
        [id3]: { children: [] },
        [id4]: { children: [] },
      });
    });

    it("move node 3 to be the peer of node 4 succeeds", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode(id1, 0, "Task 2", USER);
      const id3 = koso.insertNode(id1, 1, "Task 3", USER);
      const id4 = koso.insertNode(id1, 2, "Task 4", USER);

      koso.moveNode(id3, id1, 1, id1, 3);

      expect(koso.yGraph.toJSON()).toMatchObject({
        root: { children: [id1] },
        [id1]: { children: [id2, id4, id3] },
        [id2]: { children: [] },
        [id3]: { children: [] },
        [id4]: { children: [] },
      });
    });
  });

  describe("setTaskName", () => {
    it("set node 2's name succeeds", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode("root", 0, "Task 2", USER);

      koso.setTaskName(id2, "Edited Task 2");

      expect(koso.yGraph.toJSON()).toMatchObject({
        root: { name: "Root" },
        [id1]: { name: "Task 1" },
        [id2]: { name: "Edited Task 2" },
      });
    });
  });

  describe("setAssignee", () => {
    it("set node 2's assignee succeeds", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode("root", 0, "Task 2", USER);

      koso.setAssignee(id2, USER);

      expect(koso.yGraph.toJSON()).toMatchObject({
        root: { assignee: null },
        [id1]: { assignee: null },
        [id2]: { assignee: "t@koso.app" },
      });
    });
  });

  describe("setReporter", () => {
    it("set node 2's reporter succeeds", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode("root", 0, "Task 2", USER);

      koso.setReporter(id2, {
        email: "new@koso.app",
        name: "New Test User",
        picture: "",
        exp: 0,
      });

      expect(koso.yGraph.toJSON()).toMatchObject({
        root: { reporter: null },
        [id1]: { reporter: "t@koso.app" },
        [id2]: { reporter: "new@koso.app" },
      });
    });
  });

  describe("setTaskStatus", () => {
    it("set node 2's reporter succeeds", () => {
      const id1 = koso.insertNode("root", 0, "Task 1", USER);
      const id2 = koso.insertNode("root", 0, "Task 2", USER);

      koso.setTaskStatus(id2, "Done");

      expect(koso.yGraph.toJSON()).toMatchObject({
        root: { status: null },
        [id1]: { status: null },
        [id2]: { status: "Done" },
      });
    });
  });
});
