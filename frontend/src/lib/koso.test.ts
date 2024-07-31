import { beforeEach, describe, expect, it } from "vitest";
import * as Y from "yjs";
import { Koso, Node } from "./koso";

function addItem(koso: Koso, id: string, name: string, children: string[]) {
  koso.upsert({ id, name, children, reporter: "t@koso.app", assignee: null });
}

describe("Koso tests", () => {
  let koso: Koso;
  beforeEach(() => {
    koso = new Koso("project-id", new Y.Doc());
    koso.handleClientMessage(() => {});
  });

  describe("getRoots", () => {
    it("empty graph returns empty set", () => {
      expect(koso.getRoots()).toStrictEqual(new Set([]));
    });

    it("graph with one task returns one root", () => {
      addItem(koso, "1", "Task 1", []);
      expect(koso.getRoots()).toStrictEqual(new Set(["1"]));
    });

    it("graph with two tasks and one root returns one root", () => {
      addItem(koso, "1", "Task 1", ["2"]);
      addItem(koso, "2", "Task 2", []);
      expect(koso.getRoots()).toStrictEqual(new Set(["1"]));
    });

    it("graph with two roots returns two roots", () => {
      addItem(koso, "1", "Task 1", []);
      addItem(koso, "2", "Task 2", []);
      expect(koso.getRoots()).toStrictEqual(new Set(["1", "2"]));
    });
  });

  describe("getTask", () => {
    it("retrieves task 2", () => {
      addItem(koso, "1", "Task 1", ["2"]);
      addItem(koso, "2", "Task 2", []);
      expect(koso.getTask("2")).toStrictEqual({
        id: "2",
        name: "Task 2",
        children: [],
        reporter: "t@koso.app",
        assignee: null,
      });
    });

    it("invalid task id throws an exception", () => {
      addItem(koso, "1", "Task 1", []);
      addItem(koso, "2", "Task 2", []);
      expect(() => koso.getTask("3")).toThrow();
    });
  });

  describe("getChildren", () => {
    it("retrieves task 1's children", () => {
      addItem(koso, "1", "Task 1", ["2"]);
      addItem(koso, "2", "Task 2", []);
      expect(koso.getChildren("1")).toStrictEqual(["2"]);
    });

    it("retrieves empty list of children for leaf task", () => {
      addItem(koso, "1", "Task 1", ["2"]);
      addItem(koso, "2", "Task 2", []);
      expect(koso.getChildren("2")).toStrictEqual([]);
    });

    it("invalid task id throws an exception", () => {
      addItem(koso, "1", "Task 1", []);
      addItem(koso, "2", "Task 2", []);
      expect(() => koso.getChildren("3")).toThrow();
    });
  });

  describe("toNodes", () => {
    it("empty graph renders successfully", () => {
      expect(koso.toNodes()).toStrictEqual([]);
    });

    it("graph with one root node renders successfully", () => {
      addItem(koso, "1", "Task 1", []);
      expect(koso.toNodes()).toStrictEqual([new Node(["1"])]);
    });

    it("populated graph renders successfully", () => {
      addItem(koso, "1", "Task 1", ["2"]);
      addItem(koso, "2", "Task 2", []);
      expect(koso.toNodes()).toStrictEqual([
        new Node(["1"]),
        new Node(["1", "2"]),
      ]);
    });
  });

  describe("graph", () => {
    it("empty graph renders successfully", () => {
      expect(koso.yGraph.toJSON()).toStrictEqual({});
    });

    it("graph with one root node renders to json successfully", () => {
      addItem(koso, "1", "Task 1", []);
      expect(koso.yGraph.toJSON()).toStrictEqual({
        "1": {
          id: "1",
          name: "Task 1",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("populated graph renders to json successfully", () => {
      addItem(koso, "1", "Task 1", ["2"]);
      addItem(koso, "2", "Task 2", []);
      expect(koso.yGraph.toJSON()).toStrictEqual({
        "1": {
          id: "1",
          name: "Task 1",
          children: ["2"],
          reporter: "t@koso.app",
          assignee: null,
        },
        "2": {
          id: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("reparent root node 2 to root node 1 succeeds", () => {
      addItem(koso, "1", "Task 1", []);
      addItem(koso, "2", "Task 2", []);

      koso.addNode("2", "1", 0);

      expect(koso.yGraph.toJSON()).toStrictEqual({
        "1": {
          id: "1",
          name: "Task 1",
          children: ["2"],
          reporter: "t@koso.app",
          assignee: null,
        },
        "2": {
          id: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("unparent node 2 from node 1 succeeds", () => {
      addItem(koso, "1", "Task 1", ["2"]);
      addItem(koso, "2", "Task 2", []);

      koso.removeNode("2", "1");

      expect(koso.yGraph.toJSON()).toStrictEqual({
        "1": {
          id: "1",
          name: "Task 1",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        "2": {
          id: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("reparent root node 3 to node 1 as a peer of node 2 succeeds", () => {
      addItem(koso, "1", "Task 1", ["2"]);
      addItem(koso, "2", "Task 2", []);
      addItem(koso, "3", "Task 3", []);

      koso.addNode("3", "1", 1);

      expect(koso.yGraph.toJSON()).toStrictEqual({
        "1": {
          id: "1",
          name: "Task 1",
          children: ["2", "3"],
          reporter: "t@koso.app",
          assignee: null,
        },
        "2": {
          id: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        "3": {
          id: "3",
          name: "Task 3",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("reparent root node 3 to node 1 as the immediate child succeeds", () => {
      addItem(koso, "1", "Task 1", ["2"]);
      addItem(koso, "2", "Task 2", []);
      addItem(koso, "3", "Task 3", []);

      koso.addNode("3", "1", 0);

      expect(koso.yGraph.toJSON()).toStrictEqual({
        "1": {
          id: "1",
          name: "Task 1",
          children: ["3", "2"],
          reporter: "t@koso.app",
          assignee: null,
        },
        "2": {
          id: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        "3": {
          id: "3",
          name: "Task 3",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("editing node 2's name succeeds", () => {
      addItem(koso, "1", "Task 1", []);
      addItem(koso, "2", "Task 2", []);

      koso.editTaskName("2", "Edited Task 2");

      expect(koso.yGraph.toJSON()).toStrictEqual({
        "1": {
          id: "1",
          name: "Task 1",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        "2": {
          id: "2",
          name: "Edited Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("move node 4 to be a child of node 3 removes it as a child from node 1", () => {
      addItem(koso, "1", "Task 1", ["2", "3", "4"]);
      addItem(koso, "2", "Task 2", []);
      addItem(koso, "3", "Task 3", []);
      addItem(koso, "4", "Task 4", []);

      koso.moveNode("4", "1", 2, "3", 0);

      expect(koso.yGraph.toJSON()).toStrictEqual({
        "1": {
          id: "1",
          name: "Task 1",
          children: ["2", "3"],
          reporter: "t@koso.app",
          assignee: null,
        },
        "2": {
          id: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        "3": {
          id: "3",
          name: "Task 3",
          children: ["4"],
          reporter: "t@koso.app",
          assignee: null,
        },
        "4": {
          id: "4",
          name: "Task 4",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("move node 4 to be the peer of node 2 succeeds", () => {
      addItem(koso, "1", "Task 1", ["2", "3", "4"]);
      addItem(koso, "2", "Task 2", []);
      addItem(koso, "3", "Task 3", []);
      addItem(koso, "4", "Task 4", []);

      koso.moveNode("4", "1", 2, "1", 1);

      expect(koso.yGraph.toJSON()).toStrictEqual({
        "1": {
          id: "1",
          name: "Task 1",
          children: ["2", "4", "3"],
          reporter: "t@koso.app",
          assignee: null,
        },
        "2": {
          id: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        "3": {
          id: "3",
          name: "Task 3",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        "4": {
          id: "4",
          name: "Task 4",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("move node 3 to be the peer of node 4 succeeds", () => {
      addItem(koso, "1", "Task 1", ["2", "3", "4"]);
      addItem(koso, "2", "Task 2", []);
      addItem(koso, "3", "Task 3", []);
      addItem(koso, "4", "Task 4", []);

      koso.moveNode("3", "1", 1, "1", 3);

      expect(koso.yGraph.toJSON()).toStrictEqual({
        "1": {
          id: "1",
          name: "Task 1",
          children: ["2", "4", "3"],
          reporter: "t@koso.app",
          assignee: null,
        },
        "2": {
          id: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        "3": {
          id: "3",
          name: "Task 3",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        "4": {
          id: "4",
          name: "Task 4",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("insert node creates a new untitled task", () => {
      addItem(koso, "1", "Task 1", ["B", "3"]);
      addItem(koso, "B", "Task B", []);
      addItem(koso, "3", "Task 3", []);

      koso.insertNode("1", 2, {
        email: "test@koso.app",
        exp: 0,
        name: "Test",
        picture: "",
      });

      expect(koso.yGraph.toJSON()).toStrictEqual({
        "1": {
          id: "1",
          name: "Task 1",
          children: ["B", "3", "4"],
          reporter: "t@koso.app",
          assignee: null,
        },
        B: {
          id: "B",
          name: "Task B",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        "3": {
          id: "3",
          name: "Task 3",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        "4": {
          id: "4",
          name: "Untitled",
          children: [],
          assignee: null,
          reporter: "test@koso.app",
        },
      });
    });
  });
});
