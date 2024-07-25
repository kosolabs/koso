import { describe, expect, it } from "vitest";
import * as Y from "yjs";
import { Koso } from "./koso";

const pId = "something";
function addItem(koso: Koso, id: string, name: string, children: string[]) {
  koso.yGraph.set(
    id,
    new Y.Map<string | Y.Array<string>>([
      ["id", id],
      ["name", name],
      ["children", Y.Array.from(children)],
    ]),
  );
}

describe("Koso tests", () => {
  it("empty graph renders successfully", () => {
    const koso = new Koso(pId, new Y.Doc());
    expect(koso.toJSON()).toStrictEqual({});
  });

  it("graph with one root node renders to json successfully", () => {
    const koso = new Koso(pId, new Y.Doc());
    addItem(koso, "1", "Task 1", []);
    expect(koso.toJSON()).toStrictEqual({
      "1": { id: "1", name: "Task 1", children: [] },
    });
  });

  it("populated graph renders to json successfully", () => {
    const koso = new Koso(pId, new Y.Doc());
    addItem(koso, "1", "Task 1", ["2"]);
    addItem(koso, "2", "Task 2", []);
    expect(koso.toJSON()).toStrictEqual({
      "1": { id: "1", name: "Task 1", children: ["2"] },
      "2": { id: "2", name: "Task 2", children: [] },
    });
  });

  it("reparent root node 2 to root node 1 succeeds", () => {
    const koso = new Koso(pId, new Y.Doc());
    addItem(koso, "1", "Task 1", []);
    addItem(koso, "2", "Task 2", []);

    koso.addNode("2", "1", 0);

    expect(koso.toJSON()).toStrictEqual({
      "1": { id: "1", name: "Task 1", children: ["2"] },
      "2": { id: "2", name: "Task 2", children: [] },
    });
  });

  it("unparent node 2 from node 1 succeeds", () => {
    const koso = new Koso(pId, new Y.Doc());
    addItem(koso, "1", "Task 1", ["2"]);
    addItem(koso, "2", "Task 2", []);

    koso.removeNode("2", "1");

    expect(koso.toJSON()).toStrictEqual({
      "1": { id: "1", name: "Task 1", children: [] },
      "2": { id: "2", name: "Task 2", children: [] },
    });
  });

  it("reparent root node 3 to node 1 as a peer of node 2 succeeds", () => {
    const koso = new Koso(pId, new Y.Doc());
    addItem(koso, "1", "Task 1", ["2"]);
    addItem(koso, "2", "Task 2", []);
    addItem(koso, "3", "Task 3", []);

    koso.addNode("3", "1", 1);

    expect(koso.toJSON()).toStrictEqual({
      "1": { id: "1", name: "Task 1", children: ["2", "3"] },
      "2": { id: "2", name: "Task 2", children: [] },
      "3": { id: "3", name: "Task 3", children: [] },
    });
  });

  it("reparent root node 3 to node 1 as the immediate child succeeds", () => {
    const koso = new Koso(pId, new Y.Doc());
    addItem(koso, "1", "Task 1", ["2"]);
    addItem(koso, "2", "Task 2", []);
    addItem(koso, "3", "Task 3", []);

    koso.addNode("3", "1", 0);

    expect(koso.toJSON()).toStrictEqual({
      "1": { id: "1", name: "Task 1", children: ["3", "2"] },
      "2": { id: "2", name: "Task 2", children: [] },
      "3": { id: "3", name: "Task 3", children: [] },
    });
  });

  it("editing node 2's name succeeds", () => {
    const koso = new Koso(pId, new Y.Doc());
    addItem(koso, "1", "Task 1", []);
    addItem(koso, "2", "Task 2", []);

    koso.editTaskName("2", "Edited Task 2");

    expect(koso.toJSON()).toStrictEqual({
      "1": { id: "1", name: "Task 1", children: [] },
      "2": { id: "2", name: "Edited Task 2", children: [] },
    });
  });

  it("move node 4 to be a child of node 3 removes it as a child from node 1", () => {
    const koso = new Koso(pId, new Y.Doc());
    addItem(koso, "1", "Task 1", ["2", "3", "4"]);
    addItem(koso, "2", "Task 2", []);
    addItem(koso, "3", "Task 3", []);
    addItem(koso, "4", "Task 4", []);

    koso.moveNode("4", "1", 2, "3", 0);

    expect(koso.toJSON()).toStrictEqual({
      "1": { id: "1", name: "Task 1", children: ["2", "3"] },
      "2": { id: "2", name: "Task 2", children: [] },
      "3": { id: "3", name: "Task 3", children: ["4"] },
      "4": { id: "4", name: "Task 4", children: [] },
    });
  });

  it("move node 4 to be the peer of node 2 succeeds", () => {
    const koso = new Koso(pId, new Y.Doc());
    addItem(koso, "1", "Task 1", ["2", "3", "4"]);
    addItem(koso, "2", "Task 2", []);
    addItem(koso, "3", "Task 3", []);
    addItem(koso, "4", "Task 4", []);

    koso.moveNode("4", "1", 2, "1", 1);

    expect(koso.toJSON()).toStrictEqual({
      "1": { id: "1", name: "Task 1", children: ["2", "4", "3"] },
      "2": { id: "2", name: "Task 2", children: [] },
      "3": { id: "3", name: "Task 3", children: [] },
      "4": { id: "4", name: "Task 4", children: [] },
    });
  });

  it("move node 3 to be the peer of node 4 succeeds", () => {
    const koso = new Koso(pId, new Y.Doc());
    addItem(koso, "1", "Task 1", ["2", "3", "4"]);
    addItem(koso, "2", "Task 2", []);
    addItem(koso, "3", "Task 3", []);
    addItem(koso, "4", "Task 4", []);

    koso.moveNode("3", "1", 1, "1", 3);

    expect(koso.toJSON()).toStrictEqual({
      "1": { id: "1", name: "Task 1", children: ["2", "4", "3"] },
      "2": { id: "2", name: "Task 2", children: [] },
      "3": { id: "3", name: "Task 3", children: [] },
      "4": { id: "4", name: "Task 4", children: [] },
    });
  });

  it("insert node creates a new untitled task", () => {
    const koso = new Koso(pId, new Y.Doc());
    addItem(koso, "1", "Task 1", ["B", "3"]);
    addItem(koso, "B", "Task B", []);
    addItem(koso, "3", "Task 3", []);

    koso.insertNode("1", 2);

    expect(koso.toJSON()).toStrictEqual({
      "1": { id: "1", name: "Task 1", children: ["B", "3", "4"] },
      B: { id: "B", name: "Task B", children: [] },
      "3": { id: "3", name: "Task 3", children: [] },
      "4": { id: "4", name: "Untitled", children: [] },
    });
  });
});
