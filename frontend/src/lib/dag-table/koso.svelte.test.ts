import type { User } from "$lib/auth.svelte";
import { List, Set } from "immutable";
import * as encoding from "lib0/encoding";
import { uuidv4 } from "lib0/random.js";
import { beforeEach, describe, expect, it } from "vitest";
import * as Y from "yjs";
import { Koso, Node } from ".";
import { type TaskBuilder } from "../../../tests/utils";

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
          kind: task.kind ?? null,
          url: task.url ?? null,
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
          kind: null,
          url: null,
        });
      }
    });
  };

  beforeEach((context) => {
    const cleanup = $effect.root(() => {
      koso = new Koso("project-id-" + uuidv4(), new Y.Doc());
      koso.setSendAndSync(() => {});
      koso.receive(EMPTY_SYNC_RESPONSE);
    });
    context.onTestFinished(() => cleanup());
  });

  describe("link", () => {
    it("links two nodes successfully", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
      ]);

      koso.link("2", "1", 0);

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { id: "1", children: ["2"] },
        ["2"]: { id: "2", children: [] },
      });
    });

    it("linking a node to itself throws an error", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1" },
      ]);

      expect(() => koso.link("1", "1", 0)).toThrow();
    });

    it("linking a node to its parent throws an error", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);

      expect(() => koso.link("2", "1", 0)).toThrow();
    });

    it("linking a node to its child throws an error", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);

      expect(() => koso.link("1", "2", 0)).toThrow();
    });

    it("linking a node to a non-existent node throws an error", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1" },
      ]);

      expect(() => koso.link("1", "non-existent", 0)).toThrow();
    });

    it("linking a non-existent node to an existing node throws an error", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1" },
      ]);

      expect(() => koso.link("non-existent", "1", 0)).toThrow();
    });

    it("links a task after another task successfully", () => {
      init([
        { id: "root", name: "Root", children: ["1", "3"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
        { id: "3", name: "Task 3" },
      ]);

      koso.link("2", "root", 1);

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1", "2", "3"] },
        ["1"]: { id: "1", children: [] },
        ["2"]: { id: "2", children: [] },
        ["3"]: { id: "3", children: [] },
      });
    });

    it("links a task as the last task successfully", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
        { id: "3", name: "Task 3" },
      ]);

      koso.link("3", "root", 2);

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1", "2", "3"] },
      });
    });

    it("links a not started task at the top of the done stack", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2", "3", "4"] },
        { id: "1", name: "Task 1", status: "In Progress" },
        { id: "2", name: "Task 2", status: "In Progress" },
        { id: "3", name: "Task 3" },
        { id: "4", name: "Task 4", status: "Done" },
        { id: "l", name: "Link Task" },
      ]);

      koso.link("l", "root");

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1", "2", "3", "l", "4"] },
      });
    });

    it("links an in progress task at the bottom of the in progress stack", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2", "3", "4"] },
        { id: "1", name: "Task 1", status: "In Progress" },
        { id: "2", name: "Task 2", status: "In Progress" },
        { id: "3", name: "Task 3" },
        { id: "4", name: "Task 4" },
        { id: "l", name: "Link Task", status: "In Progress" },
      ]);

      koso.link("l", "root");

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1", "2", "l", "3", "4"] },
      });
    });

    it("links a done task at the top of the done stack", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2", "3", "4"] },
        { id: "1", name: "Task 1", status: "In Progress" },
        { id: "2", name: "Task 2" },
        { id: "3", name: "Task 3", status: "Done" },
        { id: "4", name: "Task 4", status: "Done" },
        { id: "l", name: "Link Task", status: "Done" },
      ]);

      koso.link("l", "root");

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1", "2", "l", "3", "4"] },
      });
    });
  });

  describe("unlink", () => {
    it("unlinks a task successfully", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);

      koso.unlink("2", "1");

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { id: "1", children: [] },
        ["2"]: { id: "2", children: [] },
      });
    });

    it("unlinking a non-existent task throws an error", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1" },
      ]);

      expect(() => koso.unlink("non-existent", "1")).toThrow();
    });

    it("unlinking a task with multiple parents succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: ["3"] },
        { id: "2", name: "Task 2", children: ["3"] },
        { id: "3", name: "Task 3" },
      ]);

      koso.unlink("3", "1");

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1", "2"] },
        ["1"]: { id: "1", children: [] },
        ["2"]: { id: "2", children: ["3"] },
        ["3"]: { id: "3", children: [] },
      });
    });

    it("unlinking a task with non-existent parent throws an error", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1" },
      ]);

      expect(() => koso.unlink("1", "non-existent")).toThrow();
    });

    it("unlinking a task among multiple peers succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2", "3"] },
        { id: "2", name: "Task 2" },
        { id: "3", name: "Task 3" },
      ]);

      koso.unlink("2", "1");

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { id: "1", children: ["3"] },
        ["2"]: { id: "2", children: [] },
        ["3"]: { id: "3", children: [] },
      });
    });

    it("unlinking a canonical plugin container throws an error", () => {
      init([
        { id: "root", name: "Root", children: ["github"] },
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
          children: ["1"],
        },
        { id: "1", name: "Task 1", kind: "github_pr" },
      ]);

      expect(() => koso.unlink("github", "root")).toThrow();
      expect(() => koso.unlink("github_pr", "github")).toThrow();
      expect(() => koso.unlink("2", "github_pr")).toThrow();
    });

    it("unlinking a non-canonical plugin task succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["github", "github_pr", "1"] },
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
          children: ["1"],
        },
        { id: "1", name: "Task 1", kind: "github_pr" },
        { id: "2", name: "Task 2", children: ["github", "1"] },
      ]);

      koso.unlink("github_pr", "root");
      koso.unlink("1", "root");
      koso.unlink("1", "2");
      koso.unlink("github", "2");

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["github"] },
        ["github"]: { id: "github", children: ["github_pr"] },
        ["github_pr"]: { id: "github_pr", children: ["1"] },
        ["2"]: { id: "2", children: [] },
      });
    });
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
          kind: null,
          url: null,
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
          kind: null,
          url: null,
        },
      });
    });

    it("inserting a child of a plugin container throws", () => {
      init([
        { id: "root", name: "Root", children: ["github", "github_pr", "1"] },
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
          children: ["1"],
        },
        { id: "1", name: "Task 1", kind: "github_pr" },
      ]);

      expect(() =>
        koso.insertNode(Node.parse("github"), 0, USER, "Task 2"),
      ).toThrow();
      expect(() =>
        koso.insertNode(Node.parse("github/github_pr"), 0, USER, "Task 2"),
      ).toThrow();
    });

    it("inserting a child of a plugin task throws", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["github", "github_pr", "1"],
        },
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
          children: ["1"],
        },
        { id: "1", name: "Task 1", kind: "github_pr" },
      ]);

      expect(() =>
        koso.insertNode(Node.parse("github/github_pr/1"), 0, USER, "Task 2"),
      ).toThrow();
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
        koso.linkNode(Node.parse("1"), Node.parse("github"), 0),
      ).toThrow();
      expect(() =>
        koso.linkNode(Node.parse("1"), Node.parse("github/github_pr"), 0),
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
        koso.linkNode(Node.parse("1"), Node.parse("github/github_pr/2"), 0),
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
      koso.linkNode(Node.parse("github/github_pr/2"), Node.parse("1"), 0);
      koso.linkNode(Node.parse("github/github_pr"), Node.parse("1"), 0);
      koso.linkNode(Node.parse("github"), Node.parse("1"), 0);
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
        koso.moveNode(Node.parse("github"), Node.parse("1"), 0),
      ).toThrow();
      expect(() =>
        koso.moveNode(Node.parse("github/github_pr"), Node.parse("1"), 0),
      ).toThrow();
      expect(() =>
        koso.moveNode(Node.parse("github/github_pr/2"), Node.parse("1"), 0),
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
      koso.moveNode(Node.parse("2"), Node.parse("1"), 0);
      koso.moveNode(Node.parse("github_pr"), Node.parse("1"), 0);
      koso.moveNode(Node.parse("3/github"), Node.parse("1"), 0);
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
      koso.moveNode(Node.parse("github"), Node.parse("root"), 0);
      expect(koso.toJSON()).toMatchObject({
        root: { children: ["github", "1"] },
      });
    });

    it("reorder canonical plugin sub-container succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1", "github"] },
        { id: "1", name: "Task 1", children: [] },
        {
          id: "github",
          name: "Github",
          kind: "github",
          children: ["github_pr", "other_plugin"],
        },
        { id: "github_pr", name: "Github PR", kind: "github_pr" },
        { id: "other_plugin", name: "Other", kind: "other_plugin" },
        { id: "2", name: "Some PR", kind: "github_pr" },
        { id: "3", name: "Some PR", kind: "github_pr" },
      ]);
      koso.moveNode(Node.parse("github/other_plugin"), Node.parse("github"), 0);
      expect(koso.toJSON()).toMatchObject({
        ["github"]: { children: ["other_plugin", "github_pr"] },
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
      koso.moveNode(
        Node.parse("github/github_pr/3"),
        Node.parse("github/github_pr"),
        0,
      );
      expect(koso.toJSON()).toMatchObject({
        ["github_pr"]: { children: ["3", "2"] },
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

    it("delete canonical plugin task/container throws", () => {
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
          children: ["2", "3", "github_pr_other"],
        },
        {
          id: "github_pr_other",
          name: "Github PR Other",
          kind: "github_pr_other",
          children: ["3"],
        },
        { id: "2", name: "Some PR", kind: "github_pr" },
        { id: "3", name: "Some Other PR", kind: "github_pr_other" },
      ]);
      expect(() => koso.deleteNode(Node.parse("github"))).toThrow();
      expect(() => koso.deleteNode(Node.parse("github/github_pr"))).toThrow();
      expect(() => koso.deleteNode(Node.parse("github/github_pr/2"))).toThrow();
      expect(() =>
        koso.deleteNode(Node.parse("github/github_pr/github_pr_other")),
      ).toThrow();
      expect(() =>
        koso.deleteNode(Node.parse("github/github_pr/github_pr_other/3")),
      ).toThrow();
    });

    it("delete non-canonical plugin task/container succeeds", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["1", "github", "githubfoo", "github_pr_other", "4", "5"],
        },
        {
          id: "1",
          name: "Task 1",
          children: ["github", "github_pr", "github_pr_other", "2", "3"],
        },
        {
          id: "github",
          name: "Github",
          kind: "github",
          children: ["github_pr", "github_pr_other", "githubfoo"],
        },
        {
          id: "github_pr",
          name: "Github PR",
          kind: "github_pr",
          children: ["2", "3", "github_pr_other", "githubfoo"],
        },
        {
          id: "github_pr_other",
          name: "Github PR Other",
          kind: "github_pr_other",
          children: ["3"],
        },
        {
          id: "githubfoo",
          name: "Github Foo",
          kind: "githubfoo",
        },
        {
          id: "2",
          name: "Some PR",
          kind: "github_pr",
          children: ["github_pr_other"],
        },
        { id: "3", name: "Some Other PR", kind: "github_pr_other" },
        { id: "4", name: "Some Rollup task", kind: "Rollup" },
        { id: "5", name: "Some Juggled task", kind: "Juggled" },
      ]);
      koso.deleteNode(Node.parse("1/github"));
      koso.deleteNode(Node.parse("1/github_pr"));
      koso.deleteNode(Node.parse("1/github_pr_other"));
      koso.deleteNode(Node.parse("1/2"));
      koso.deleteNode(Node.parse("1/3"));
      koso.deleteNode(Node.parse("github/github_pr_other"));
      koso.deleteNode(Node.parse("github/github_pr/2/github_pr_other"));
      koso.deleteNode(Node.parse("github/githubfoo"));
      koso.deleteNode(Node.parse("github_pr/githubfoo"));
      koso.deleteNode(Node.parse("github_pr_other"));
      koso.deleteNode(Node.parse("4"));
      koso.deleteNode(Node.parse("5"));

      expect(koso.toJSON()).toMatchObject({
        ["1"]: { id: "1", children: [] },
      });
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
          kind: "github",
          url: "http://example.com/foo/bar",
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
        kind: "github",
        url: "http://example.com/foo/bar",
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

      expect(koso.tasks.map((task) => task.toJSON())).toMatchObject([
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

    it("parents of 3 are root and 1", () => {
      expect(koso.getParents("3")).toEqual(["root", "1"]);
    });

    it("parents of 4 are 3 and 2", () => {
      expect(koso.getParents("4")).toEqual(["3", "2"]);
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

    it("task with no children and null status time has 0 of 1 progress", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: [] },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 0,
        done: 0,
        total: 1,
        lastStatusTime: 0,
      });
    });

    it("task with no children and status time of now has 0 of 1 progress", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: [], statusTime: now },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 0,
        done: 0,
        total: 1,
        lastStatusTime: now,
      });
    });

    it("parent task with all children done has 2 of 2 progress", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "3"] },
        { id: "2", status: "Done", statusTime: 0 },
        { id: "3", status: "Done", statusTime: now },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 0,
        done: 2,
        total: 2,
        lastStatusTime: now,
      });
    });

    it("parent task with one child in progress and one done has 1 of 2 progress", () => {
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

    it("parent task with all children not started has 0 of 2 progress", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "3"] },
        { id: "2", status: "Not Started" },
        { id: "3", status: "Not Started", statusTime: now },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 0,
        done: 0,
        total: 2,
        lastStatusTime: now,
      });
    });

    it("parent task with mixed children statuses has correct progress", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "3", "4"] },
        { id: "2", status: "Not Started" },
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

    it("parent task with nested children has correct progress", () => {
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

    it("parent task with multiple levels of nested children has correct progress", () => {
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
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3", "t4", "t5"],
        },
      ]);

      koso.setTaskStatus(Node.parse("t2"), "Done", USER);
      const children = koso.toJSON().root.children;

      expect(children).toEqual(["t1", "t3", "t4", "t5", "t2"]);
    });

    it("setting a task to In Progress moves task to the top", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3", "t4", "t5"],
        },
      ]);

      koso.setTaskStatus(Node.parse("t4"), "In Progress", USER);
      const children = koso.toJSON().root.children;

      expect(children).toEqual(["t4", "t1", "t2", "t3", "t5"]);
    });

    it("setting task to In Progress moves next to the last In Progress task", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3", "t4", "t5"],
        },
        { id: "t2", status: "In Progress" },
        { id: "t5", status: "Done" },
      ]);

      koso.setTaskStatus(Node.parse("t4"), "In Progress", USER);
      const children = koso.toJSON().root.children;

      expect(children).toEqual(["t1", "t2", "t4", "t3", "t5"]);
    });

    it("setting task to Done moves next to the first Done task", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3", "t4", "t5"],
        },
        { id: "t1", status: "In Progress" },
        { id: "t4", status: "Done" },
      ]);

      koso.setTaskStatus(Node.parse("t2"), "Done", USER);
      const children = koso.toJSON().root.children;

      expect(children).toEqual(["t1", "t3", "t2", "t4", "t5"]);
    });

    it("setting a Done task to In Progress moves up", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3", "t4", "t5"],
        },
        { id: "t1", status: "In Progress" },
        { id: "t4", status: "Done" },
        { id: "t5", status: "Done" },
      ]);

      koso.setTaskStatus(Node.parse("t5"), "In Progress", USER);
      const children = koso.toJSON().root.children;

      expect(children).toEqual(["t1", "t5", "t2", "t3", "t4"]);
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
      koso.expanded = Set([Node.parse("t4"), Node.parse("t6")]);

      let node: Node | null = Node.parse("t2");
      node = koso.getPrevLink(node);
      expect(node).toEqual(Node.parse("t6/t2"));
      if (!node) throw new Error();

      node = koso.getPrevLink(node);
      expect(node).toEqual(Node.parse("t4/t2"));
      if (!node) throw new Error();

      node = koso.getPrevLink(node);
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
      koso.expanded = Set([Node.parse("t4")]);

      let node: Node | null = Node.parse("t2");
      node = koso.getPrevLink(node);
      expect(node).toEqual(Node.parse("t4/t2"));
      if (!node) throw new Error();

      node = koso.getPrevLink(node);
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

      expect(koso.getPrevLink(Node.parse("t2"))).toBeNull();
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
      koso.expanded = Set([Node.parse("t4"), Node.parse("t6")]);

      let node: Node | null = Node.parse("t2");
      node = koso.getNextLink(node);
      expect(node).toEqual(Node.parse("t4/t2"));
      if (!node) throw new Error();

      node = koso.getNextLink(node);
      expect(node).toEqual(Node.parse("t6/t2"));
      if (!node) throw new Error();

      node = koso.getNextLink(node);
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
      koso.expanded = Set([Node.parse("t4")]);

      let node: Node | null = Node.parse("t2");
      node = koso.getNextLink(node);
      expect(node).toEqual(Node.parse("t4/t2"));
      if (!node) throw new Error();

      node = koso.getNextLink(node);
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

      expect(koso.getNextLink(Node.parse("t2"))).toBeNull();
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

      expect(koso.getNextLink(Node.parse("t2"))).toBeNull();
    });
  });
});
