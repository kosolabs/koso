import type { Kind } from "$lib/yproxy";
import { List, Set } from "immutable";
import { uuidv4 } from "lib0/random.js";
import { beforeEach, describe, expect, it } from "vitest";
import * as Y from "yjs";
import { Koso, Node } from ".";
import { EMPTY_SYNC_RESPONSE, type TaskBuilder } from "../../../tests/utils";
import { PlanningContext } from "./planning-context.svelte";

describe("PlanningContext tests", () => {
  let root: Node;
  let koso: Koso;
  let planningCtx: PlanningContext;

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
          desc: null,
          children: task.children ?? [],
          assignee: task.assignee ?? null,
          reporter: task.reporter ?? null,
          status: task.status ?? null,
          statusTime: task.statusTime ?? null,
          kind: (task.kind as Kind) ?? null,
          url: task.url ?? null,
        });
      }
      for (const taskId of remainingTaskIds) {
        koso.upsert({
          id: taskId,
          num: taskId,
          name: `Task ${taskId}`,
          desc: null,
          children: [],
          assignee: null,
          reporter: null,
          status: null,
          statusTime: null,
          kind: null,
          url: null,
        });
      }
    });
  };

  beforeEach((context) => {
    const cleanup = $effect.root(() => {
      koso = new Koso("project-id-" + uuidv4(), new Y.Doc());
      planningCtx = new PlanningContext(koso);
      root = planningCtx.root;
      koso.setSendAndSync(() => {});
      koso.receive(EMPTY_SYNC_RESPONSE);
    });
    context.onTestFinished(() => cleanup());
  });

  describe("linkNode", () => {
    it("link node 2 to node 1 succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
      ]);
      planningCtx.linkNode(Node.parse("2"), Node.parse("1"), 0);

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
        planningCtx.linkNode(Node.parse("1"), Node.parse("1"), 0),
      ).toThrow();
    });

    it("link node 1 to grandchild of node 1 throws (prevent cycle)", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);
      expect(() =>
        planningCtx.linkNode(Node.parse("1"), Node.parse("2"), 0),
      ).toThrow();
    });

    it("link node 1 to plugin container throws", () => {
      init([
        { id: "root", name: "Root", children: ["1", "github"] },
        { id: "1", name: "Task 1", children: [] },
        {
          id: "github",
          name: "Github",
          kind: "github",
          children: ["github_pr"],
        },
        {
          id: "github_pr",
          name: "Github PR",
          kind: "github_pr",
          children: ["2"],
        },
        { id: "2", name: "Some PR", kind: "github_pr" },
      ]);
      expect(() =>
        planningCtx.linkNode(Node.parse("1"), Node.parse("github"), 0),
      ).toThrow();
      expect(() =>
        planningCtx.linkNode(
          Node.parse("1"),
          Node.parse("github/github_pr"),
          0,
        ),
      ).toThrow();
    });

    it("link node 1 to plugin task throws", () => {
      init([
        { id: "root", name: "Root", children: ["1", "github"] },
        { id: "1", name: "Task 1", children: [] },
        {
          id: "github",
          name: "Github",
          kind: "github",
          children: ["github_pr"],
        },
        {
          id: "github_pr",
          name: "Github PR",
          kind: "github_pr",
          children: ["2"],
        },
        { id: "2", name: "Some PR", kind: "github_pr" },
      ]);
      expect(() =>
        planningCtx.linkNode(
          Node.parse("1"),
          Node.parse("github/github_pr/2"),
          0,
        ),
      ).toThrow();
    });

    it("link plugin task/container elsewhere succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1", "github"] },
        { id: "1", name: "Task 1", children: [] },
        {
          id: "github",
          name: "Github",
          kind: "github",
          children: ["github_pr"],
        },
        {
          id: "github_pr",
          name: "Github PR",
          kind: "github_pr",
          children: ["2"],
        },
        { id: "2", name: "Some PR", kind: "github_pr" },
      ]);
      planningCtx.linkNode(
        Node.parse("github/github_pr/2"),
        Node.parse("1"),
        0,
      );
      planningCtx.linkNode(Node.parse("github/github_pr"), Node.parse("1"), 0);
      planningCtx.linkNode(Node.parse("github"), Node.parse("1"), 0);
      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1", "github"] },
        ["github"]: { children: ["github_pr"] },
        ["github_pr"]: { children: ["2"] },
        ["1"]: { id: "1", children: ["github", "github_pr", "2"] },
      });
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

      planningCtx.moveNode(Node.parse("3"), Node.parse("1"), 1);

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

      planningCtx.moveNode(Node.parse("3"), Node.parse("1"), 0);

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

      planningCtx.moveNode(Node.parse("1/4"), Node.parse("1/3"), 0);

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
        planningCtx.moveNode(Node.parse("2"), Node.parse("1"), 1),
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

      planningCtx.moveNode(Node.parse("1/4"), Node.parse("1"), 1);

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

      planningCtx.moveNode(Node.parse("1/3"), Node.parse("1"), 3);

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: ["2", "4", "3"] },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
        ["4"]: { children: [] },
      });
    });

    it("move canonical plugin task/container elsewhere throws", () => {
      init([
        { id: "root", name: "Root", children: ["1", "github"] },
        { id: "1", name: "Task 1", children: [] },
        {
          id: "github",
          name: "Github",
          kind: "github",
          children: ["github_pr"],
        },
        {
          id: "github_pr",
          name: "Github PR",
          kind: "github_pr",
          children: ["2"],
        },
        { id: "2", name: "Some PR", kind: "github_pr" },
      ]);
      expect(() =>
        planningCtx.moveNode(Node.parse("github"), Node.parse("1"), 0),
      ).toThrow();
      expect(() =>
        planningCtx.moveNode(
          Node.parse("github/github_pr"),
          Node.parse("1"),
          0,
        ),
      ).toThrow();
      expect(() =>
        planningCtx.moveNode(
          Node.parse("github/github_pr/2"),
          Node.parse("1"),
          0,
        ),
      ).toThrow();
    });

    it("move non-canonical plugin task/container succeeds", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["1", "github", "github_pr", "2"],
        },
        { id: "1", name: "Task 1" },
        { id: "3", name: "Task 3", children: ["github"] },
        {
          id: "github",
          name: "Github",
          kind: "github",
          children: ["github_pr"],
        },
        {
          id: "github_pr",
          name: "Github PR",
          kind: "github_pr",
          children: ["2"],
        },
        { id: "2", name: "Some PR", kind: "github_pr" },
      ]);
      planningCtx.moveNode(Node.parse("2"), Node.parse("1"), 0);
      planningCtx.moveNode(Node.parse("github_pr"), Node.parse("1"), 0);
      planningCtx.moveNode(Node.parse("3/github"), Node.parse("1"), 0);
      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1", "github"] },
        ["github"]: { children: ["github_pr"] },
        ["github_pr"]: { children: ["2"] },
        ["1"]: { children: ["github", "github_pr", "2"] },
        ["3"]: { children: [] },
      });
    });

    it("reorder canonical plugin container succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1", "github"] },
        { id: "1", name: "Task 1", children: [] },
        {
          id: "github",
          name: "Github",
          kind: "github",
        },
      ]);
      planningCtx.moveNode(Node.parse("github"), Node.parse("root"), 0);
      expect(koso.toJSON()).toMatchObject({
        root: { children: ["github", "1"] },
      });
    });

    it("reorder canonical plugin task succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1", "github"] },
        { id: "1", name: "Task 1", children: [] },
        {
          id: "github",
          name: "Github",
          kind: "github",
          children: ["github_pr", "other_plugin"],
        },
        {
          id: "github_pr",
          name: "Github PR",
          kind: "github_pr",
          children: ["2", "3"],
        },
        { id: "2", name: "Some PR", kind: "github_pr" },
        { id: "3", name: "Some PR", kind: "github_pr" },
      ]);
      planningCtx.moveNode(
        Node.parse("github/github_pr/3"),
        Node.parse("github/github_pr"),
        0,
      );
      expect(koso.toJSON()).toMatchObject({
        ["github_pr"]: { children: ["3", "2"] },
      });
    });
  });

  describe("toNodes", () => {
    beforeEach(() => {
      planningCtx.expanded = Set([]);
    });

    it("empty doc has no nodes", () => {
      expect(planningCtx.nodes).toStrictEqual(List([root]));
    });

    it("doc with one task has one node", () => {
      init([{ id: "root", name: "Root", children: ["1"] }]);
      expect(planningCtx.nodes).toStrictEqual(List([root, Node.parse("1")]));
    });

    it("doc with non-visible tasks returns root", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", status: "Done" },
      ]);
      expect(planningCtx.nodes).toStrictEqual(List([root]));
    });

    it("doc with two tasks has two nodes", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
      ]);
      expect(planningCtx.nodes).toStrictEqual(
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
      expect(planningCtx.nodes).toStrictEqual(
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
      planningCtx.expanded = Set([Node.parse("1")]);
      expect(planningCtx.nodes).toStrictEqual(
        List([root, Node.parse("1"), Node.parse("1/3"), Node.parse("2")]),
      );
    });

    it("doc with two tasks, one linked subtask, and parent is expanded has three nodes", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);
      planningCtx.expanded = Set([Node.parse("1")]);
      expect(planningCtx.nodes).toStrictEqual(
        List([root, Node.parse("1"), Node.parse("1/2"), Node.parse("2")]),
      );
    });
  });

  describe("getPrevLink", { sequential: true }, () => {
    it("loops through expanded nodes", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3", "t4", "t5", "t6"],
        },
        { id: "t2" },
        { id: "t4", children: ["t2"] },
        { id: "t6", children: ["t2"] },
      ]);
      planningCtx.expanded = Set([Node.parse("t4"), Node.parse("t6")]);

      let node: Node | null = Node.parse("t2");
      node = planningCtx.getPrevLink(node);
      expect(node).toEqual(Node.parse("t6/t2"));
      if (!node) throw new Error();

      node = planningCtx.getPrevLink(node);
      expect(node).toEqual(Node.parse("t4/t2"));
      if (!node) throw new Error();

      node = planningCtx.getPrevLink(node);
      expect(node).toEqual(Node.parse("t2"));
    });

    it("ignores collapsed nodes", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3", "t4", "t5", "t6"],
        },
        { id: "t2" },
        { id: "t4", children: ["t2"] },
        { id: "t6", children: ["t2"] },
      ]);
      planningCtx.expanded = Set([Node.parse("t4")]);

      let node: Node | null = Node.parse("t2");
      node = planningCtx.getPrevLink(node);
      expect(node).toEqual(Node.parse("t4/t2"));
      if (!node) throw new Error();

      node = planningCtx.getPrevLink(node);
      expect(node).toEqual(Node.parse("t2"));
    });

    it("does not repeat single node", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3"],
        },
        { id: "t2" },
      ]);

      expect(planningCtx.getPrevLink(Node.parse("t2"))).toBeNull();
    });
  });

  describe("getNextLink", { sequential: true }, () => {
    it("loops through expanded nodes", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3", "t4", "t5", "t6"],
        },
        { id: "t2" },
        { id: "t4", children: ["t2"] },
        { id: "t6", children: ["t2"] },
      ]);
      planningCtx.expanded = Set([Node.parse("t4"), Node.parse("t6")]);

      let node: Node | null = Node.parse("t2");
      node = planningCtx.getNextLink(node);
      expect(node).toEqual(Node.parse("t4/t2"));
      if (!node) throw new Error();

      node = planningCtx.getNextLink(node);
      expect(node).toEqual(Node.parse("t6/t2"));
      if (!node) throw new Error();

      node = planningCtx.getNextLink(node);
      expect(node).toEqual(Node.parse("t2"));
    });

    it("ignores collapsed nodes", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3", "t4", "t5", "t6"],
        },
        { id: "t2" },
        { id: "t4", children: ["t2"] },
        { id: "t6", children: ["t2"] },
      ]);
      planningCtx.expanded = Set([Node.parse("t4")]);

      let node: Node | null = Node.parse("t2");
      node = planningCtx.getNextLink(node);
      expect(node).toEqual(Node.parse("t4/t2"));
      if (!node) throw new Error();

      node = planningCtx.getNextLink(node);
      expect(node).toEqual(Node.parse("t2"));
    });

    it("does not repeat single node", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3"],
        },
        { id: "t2" },
      ]);

      expect(planningCtx.getNextLink(Node.parse("t2"))).toBeNull();
    });
  });

  describe("getNextLink", { sequential: true }, () => {
    it("does not repeat single node", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3"],
        },
        { id: "t2" },
      ]);

      expect(planningCtx.getNextLink(Node.parse("t2"))).toBeNull();
    });
  });
});
