import { beforeEach, describe, expect, it } from "vitest";
import * as Y from "yjs";
import { Koso, Node } from "./koso";

function addItem(
  koso: Koso,
  id: string,
  num: string,
  name: string,
  children: string[],
) {
  koso.upsert({
    id,
    num,
    name,
    children,
    reporter: "t@koso.app",
    assignee: null,
    status: null,
  });
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
      addItem(koso, "id1", "1", "Task 1", []);
      expect(koso.getRoots()).toStrictEqual(new Set(["id1"]));
    });

    it("graph with two tasks and one root returns one root", () => {
      addItem(koso, "id1", "1", "Task 1", ["id2"]);
      addItem(koso, "id2", "2", "Task 2", []);
      expect(koso.getRoots()).toStrictEqual(new Set(["id1"]));
    });

    it("graph with two roots returns two roots", () => {
      addItem(koso, "id1", "1", "Task 1", []);
      addItem(koso, "id2", "2", "Task 2", []);
      expect(koso.getRoots()).toStrictEqual(new Set(["id1", "id2"]));
    });
  });

  describe("getTask", () => {
    it("retrieves task 2", () => {
      addItem(koso, "id1", "1", "Task 1", ["id2"]);
      addItem(koso, "id2", "2", "Task 2", []);
      expect(koso.getTask("id2")).toStrictEqual({
        id: "id2",
        num: "2",
        name: "Task 2",
        children: [],
        reporter: "t@koso.app",
        assignee: null,
      });
    });

    it("invalid task id throws an exception", () => {
      addItem(koso, "id1", "1", "Task 1", []);
      addItem(koso, "id2", "2", "Task 2", []);
      expect(() => koso.getTask("id3")).toThrow();
    });
  });

  describe("getChildren", () => {
    it("retrieves task 1's children", () => {
      addItem(koso, "id1", "1", "Task 1", ["id2"]);
      addItem(koso, "id2", "2", "Task 2", []);
      expect(koso.getChildren("id1")).toStrictEqual(["id2"]);
    });

    it("retrieves empty list of children for leaf task", () => {
      addItem(koso, "id1", "1", "Task 1", ["id2"]);
      addItem(koso, "id2", "2", "Task 2", []);
      expect(koso.getChildren("id2")).toStrictEqual([]);
    });

    it("invalid task id throws an exception", () => {
      addItem(koso, "id1", "1", "Task 1", []);
      addItem(koso, "id2", "2", "Task 2", []);
      expect(() => koso.getChildren("id3")).toThrow();
    });
  });

  describe("toNodes", () => {
    it("empty graph renders successfully", () => {
      expect(koso.toNodes()).toStrictEqual([]);
    });

    it("graph with one root node renders successfully", () => {
      addItem(koso, "id1", "1", "Task 1", []);
      expect(koso.toNodes()).toStrictEqual([new Node(["id1"])]);
    });

    it("populated graph renders successfully", () => {
      addItem(koso, "id1", "1", "Task 1", ["id2"]);
      addItem(koso, "id2", "2", "Task 2", []);
      expect(koso.toNodes()).toStrictEqual([
        new Node(["id1"]),
        new Node(["id1", "id2"]),
      ]);
    });
  });

  describe("graph", () => {
    it("empty graph renders successfully", () => {
      expect(koso.yGraph.toJSON()).toStrictEqual({});
    });

    it("graph with one root node renders to json successfully", () => {
      addItem(koso, "id1", "1", "Task 1", []);
      expect(koso.yGraph.toJSON()).toStrictEqual({
        id1: {
          id: "id1",
          num: "1",
          name: "Task 1",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("populated graph renders to json successfully", () => {
      addItem(koso, "id1", "1", "Task 1", ["id2"]);
      addItem(koso, "id2", "2", "Task 2", []);
      expect(koso.yGraph.toJSON()).toStrictEqual({
        id1: {
          id: "id1",
          num: "1",
          name: "Task 1",
          children: ["id2"],
          reporter: "t@koso.app",
          assignee: null,
        },
        id2: {
          id: "id2",
          num: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("reparent root node 2 to root node 1 succeeds", () => {
      addItem(koso, "id1", "1", "Task 1", []);
      addItem(koso, "id2", "2", "Task 2", []);

      koso.addNode("id2", "id1", 0);

      expect(koso.yGraph.toJSON()).toStrictEqual({
        id1: {
          id: "id1",
          num: "1",
          name: "Task 1",
          children: ["id2"],
          reporter: "t@koso.app",
          assignee: null,
        },
        id2: {
          id: "id2",
          num: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("unparent node 2 from node 1 succeeds", () => {
      addItem(koso, "id1", "1", "Task 1", ["id2"]);
      addItem(koso, "id2", "2", "Task 2", []);

      koso.removeNode("id2", "id1");

      expect(koso.yGraph.toJSON()).toStrictEqual({
        id1: {
          id: "id1",
          num: "1",
          name: "Task 1",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        id2: {
          id: "id2",
          num: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("reparent root node 3 to node 1 as a peer of node 2 succeeds", () => {
      addItem(koso, "id1", "1", "Task 1", ["id2"]);
      addItem(koso, "id2", "2", "Task 2", []);
      addItem(koso, "id3", "3", "Task 3", []);

      koso.addNode("id3", "id1", 1);

      expect(koso.yGraph.toJSON()).toStrictEqual({
        id1: {
          id: "id1",
          num: "1",
          name: "Task 1",
          children: ["id2", "id3"],
          reporter: "t@koso.app",
          assignee: null,
        },
        id2: {
          id: "id2",
          num: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        id3: {
          id: "id3",
          num: "3",
          name: "Task 3",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("reparent root node 3 to node 1 as the immediate child succeeds", () => {
      addItem(koso, "id1", "1", "Task 1", ["id2"]);
      addItem(koso, "id2", "2", "Task 2", []);
      addItem(koso, "id3", "3", "Task 3", []);

      koso.addNode("id3", "id1", 0);

      expect(koso.yGraph.toJSON()).toStrictEqual({
        id1: {
          id: "id1",
          num: "1",
          name: "Task 1",
          children: ["id3", "id2"],
          reporter: "t@koso.app",
          assignee: null,
        },
        id2: {
          id: "id2",
          num: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        id3: {
          id: "id3",
          num: "3",
          name: "Task 3",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("editing node 2's name succeeds", () => {
      addItem(koso, "id1", "1", "Task 1", []);
      addItem(koso, "id2", "2", "Task 2", []);

      koso.editTaskName("id2", "Edited Task 2");

      expect(koso.yGraph.toJSON()).toStrictEqual({
        id1: {
          id: "id1",
          num: "1",
          name: "Task 1",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        id2: {
          id: "id2",
          num: "2",
          name: "Edited Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("move node 4 to be a child of node 3 removes it as a child from node 1", () => {
      addItem(koso, "id1", "1", "Task 1", ["id2", "id3", "id4"]);
      addItem(koso, "id2", "2", "Task 2", []);
      addItem(koso, "id3", "3", "Task 3", []);
      addItem(koso, "id4", "4", "Task 4", []);

      koso.moveNode("id4", "id1", 2, "id3", 0);

      expect(koso.yGraph.toJSON()).toStrictEqual({
        id1: {
          id: "id1",
          num: "1",
          name: "Task 1",
          children: ["id2", "id3"],
          reporter: "t@koso.app",
          assignee: null,
        },
        id2: {
          id: "id2",
          num: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        id3: {
          id: "id3",
          num: "3",
          name: "Task 3",
          children: ["id4"],
          reporter: "t@koso.app",
          assignee: null,
        },
        id4: {
          id: "id4",
          num: "4",
          name: "Task 4",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("move node 4 to be the peer of node 2 succeeds", () => {
      addItem(koso, "id1", "1", "Task 1", ["id2", "id3", "id4"]);
      addItem(koso, "id2", "2", "Task 2", []);
      addItem(koso, "id3", "3", "Task 3", []);
      addItem(koso, "id4", "4", "Task 4", []);

      koso.moveNode("id4", "id1", 2, "id1", 1);

      expect(koso.yGraph.toJSON()).toStrictEqual({
        id1: {
          id: "id1",
          num: "1",
          name: "Task 1",
          children: ["id2", "id4", "id3"],
          reporter: "t@koso.app",
          assignee: null,
        },
        id2: {
          id: "id2",
          num: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        id3: {
          id: "id3",
          num: "3",
          name: "Task 3",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        id4: {
          id: "id4",
          num: "4",
          name: "Task 4",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("move node 3 to be the peer of node 4 succeeds", () => {
      addItem(koso, "id1", "1", "Task 1", ["id2", "id3", "id4"]);
      addItem(koso, "id2", "2", "Task 2", []);
      addItem(koso, "id3", "3", "Task 3", []);
      addItem(koso, "id4", "4", "Task 4", []);

      koso.moveNode("id3", "id1", 1, "id1", 3);

      expect(koso.yGraph.toJSON()).toStrictEqual({
        id1: {
          id: "id1",
          num: "1",
          name: "Task 1",
          children: ["id2", "id4", "id3"],
          reporter: "t@koso.app",
          assignee: null,
        },
        id2: {
          id: "id2",
          num: "2",
          name: "Task 2",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        id3: {
          id: "id3",
          num: "3",
          name: "Task 3",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
        id4: {
          id: "id4",
          num: "4",
          name: "Task 4",
          children: [],
          reporter: "t@koso.app",
          assignee: null,
        },
      });
    });

    it("insert node creates a new untitled task", () => {
      addItem(koso, "id1", "1", "Task 1", ["idB", "id3"]);
      addItem(koso, "idB", "2", "Task B", []);
      addItem(koso, "id3", "3", "Task 3", []);

      koso.insertNode("id1", 2, {
        email: "test@koso.app",
        exp: 0,
        name: "Test",
        picture: "",
      });

      const graph = koso.yGraph.toJSON();
      expect(Object.keys(graph)).toHaveLength(4);

      const idMatching = expect.stringMatching(/[0-9a-f-]{36}/);
      expect(graph).toStrictEqual(
        expect.objectContaining({
          id1: {
            id: "id1",
            num: "1",
            name: "Task 1",
            children: ["idB", "id3", idMatching],
            reporter: "t@koso.app",
            assignee: null,
          },
          idB: {
            id: "idB",
            num: "2",
            name: "Task B",
            children: [],
            reporter: "t@koso.app",
            assignee: null,
          },
          id3: {
            id: "id3",
            num: "3",
            name: "Task 3",
            children: [],
            reporter: "t@koso.app",
            assignee: null,
          },
        }),
      );

      // Also verify the inserted task, a child of task id1
      const insertedTaskid = graph.id1.children[2];
      const insertedTask = graph[insertedTaskid];
      expect(insertedTask.id).toStrictEqual(insertedTaskid);
      expect(graph[insertedTaskid]).toStrictEqual({
        id: idMatching,
        num: "4",
        name: "Untitled",
        children: [],
        assignee: null,
        reporter: "test@koso.app",
      });
    });
  });
});
