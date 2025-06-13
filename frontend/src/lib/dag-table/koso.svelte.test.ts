import type { User } from "$lib/users";
import { defaultTask, type Task } from "$lib/yproxy";
import { Set } from "immutable";
import { uuidv4 } from "lib0/random.js";
import { beforeEach, describe, expect, it } from "vitest";
import * as Y from "yjs";
import {
  buildTask,
  EMPTY_SYNC_RESPONSE,
  fullyPopulatedTask,
  type TaskBuilder,
} from "../../../tests/utils";
import { Koso, TaskLinkage } from "./koso.svelte";
import { Node } from "./planning-context.svelte";

const USER: User = {
  email: "t@koso.app",
  name: "Test User",
  picture: "",
};

const OTHER_USER: User = {
  email: "t2@koso.app",
  name: "Test2 User",
  picture: "",
};

describe("Koso tests", () => {
  let koso: Koso;

  const init = (tasks: TaskBuilder[]) => {
    const upsertedTaskIds = Set<string>(tasks.map((t) => t.id));
    const childTaskIds = Set<string>(tasks.flatMap((t) => t.children ?? []));
    const remainingTaskIds = childTaskIds.subtract(upsertedTaskIds);
    koso.doc.transact(() => {
      for (const task of tasks) {
        koso.upsert(buildTask(task));
      }
      for (const taskId of remainingTaskIds) {
        koso.upsert({
          ...defaultTask(),
          id: taskId,
          num: taskId,
          name: `Task ${taskId}`,
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

      koso.link(new TaskLinkage({ parentId: "1", id: "2" }), 0);

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

      expect(() =>
        koso.link(new TaskLinkage({ parentId: "1", id: "1" }), 0),
      ).toThrow();
    });

    it("linking a node to its parent throws an error", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);

      expect(() =>
        koso.link(new TaskLinkage({ parentId: "1", id: "2" }), 0),
      ).toThrow();
    });

    it("linking a node to its child throws an error", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);

      expect(() =>
        koso.link(new TaskLinkage({ parentId: "2", id: "1" }), 0),
      ).toThrow();
    });

    it("linking a node to a non-existent node throws an error", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1" },
      ]);

      expect(() =>
        koso.link(new TaskLinkage({ parentId: "non-existent", id: "1" }), 0),
      ).toThrow();
    });

    it("linking a non-existent node to an existing node throws an error", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1" },
      ]);

      expect(() =>
        koso.link(new TaskLinkage({ parentId: "1", id: "non-existent" }), 0),
      ).toThrow();
    });

    it("links a task after another task successfully", () => {
      init([
        { id: "root", name: "Root", children: ["1", "3"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
        { id: "3", name: "Task 3" },
      ]);

      koso.link(new TaskLinkage({ parentId: "root", id: "2" }), 1);

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

      koso.link(new TaskLinkage({ parentId: "root", id: "3" }), 2);

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

      koso.link(new TaskLinkage({ parentId: "root", id: "l" }));

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

      koso.link(new TaskLinkage({ parentId: "root", id: "l" }));

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

      koso.link(new TaskLinkage({ parentId: "root", id: "l" }));

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

  describe("insertTask", () => {
    it("creates a child of root", () => {
      const id1 = koso.insertTask({
        name: "Task 1",
        parent: "root",
        reporter: USER.email,
      });
      expect(koso.toJSON()).toEqual<{ [id: string]: Task }>({
        root: {
          ...defaultTask(),
          id: "root",
          num: "0",
          name: "Root",
          children: [id1],
        },
        [id1]: {
          ...defaultTask(),
          id: id1,
          num: "1",
          name: "Task 1",
          reporter: "t@koso.app",
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
        koso.insertTask({
          name: "Task 2",
          parent: "github",
          reporter: USER.email,
        }),
      ).toThrow();
      expect(() =>
        koso.insertTask({
          name: "Task 2",
          parent: "github_pr",
          reporter: USER.email,
        }),
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
        koso.insertTask({
          name: "Task 2",
          parent: "1",
          reporter: USER.email,
        }),
      ).toThrow();
    });
  });

  describe("link", () => {
    it("link node 2 to node 1 succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
      ]);
      koso.link(new TaskLinkage({ parentId: "1", id: "2" }), 0);

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
        koso.link(new TaskLinkage({ parentId: "1", id: "1" }), 0),
      ).toThrow();
    });

    it("link node 1 to grandchild of node 1 throws (prevent cycle)", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);
      expect(() =>
        koso.link(new TaskLinkage({ parentId: "2", id: "1" }), 0),
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
        koso.link(new TaskLinkage({ parentId: "github", id: "1" }), 0),
      ).toThrow();
      expect(() =>
        koso.link(
          new TaskLinkage({ parentId: "github/github_pr", id: "1" }),
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
        koso.link(
          new TaskLinkage({ parentId: "github/github_pr/2", id: "1" }),
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
      koso.link(new TaskLinkage({ parentId: "1", id: "2" }), 0);
      koso.link(new TaskLinkage({ parentId: "1", id: "github_pr" }), 0);
      koso.link(new TaskLinkage({ parentId: "1", id: "github" }), 0);
      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1", "github"] },
        ["github"]: { children: ["github_pr"] },
        ["github_pr"]: { children: ["2"] },
        ["1"]: { id: "1", children: ["github", "github_pr", "2"] },
      });
    });
  });

  describe("move", () => {
    it("move node 3 to child of node 1 as a peer of node 2 succeeds (reparent)", () => {
      init([
        { id: "root", name: "Root", children: ["1", "3"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
        { id: "3", name: "Task 3" },
      ]);

      koso.move("3", "root", "1", 1);

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

      koso.move("3", "root", "1", 0);

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

      koso.move("4", "1", "3", 0);

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

      expect(() => koso.move("2", "root", "1", 1)).toThrow();
    });

    it("move node 4 to be the peer of node 2 succeeds (reorder)", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2", "3", "4"] },
        { id: "2", name: "Task 2" },
        { id: "3", name: "Task 3" },
        { id: "4", name: "Task 4" },
      ]);

      koso.move("4", "1", "1", 1);

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

      koso.move("3", "1", "1", 3);

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
      expect(() => koso.move("github", "root", "1", 0)).toThrow();
      expect(() => koso.move("github_pr", "github", "1", 0)).toThrow();
      expect(() => koso.move("2", "github_pr", "1", 0)).toThrow();
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
      koso.move("2", "root", "1", 0);
      koso.move("github_pr", "root", "1", 0);
      koso.move("github", "3", "1", 0);
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
      koso.move("github", "root", "root", 0);
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
      koso.move("3", "github_pr", "github_pr", 0);
      expect(koso.toJSON()).toMatchObject({
        ["github_pr"]: { children: ["3", "2"] },
      });
    });
  });

  describe("delete", () => {
    it("delete node 2 from node 1 succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);

      koso.delete(Node.parse("1/2").linkage);

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

      koso.delete(Node.parse("2").linkage);

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1", "7"] },
        ["1"]: { id: "1", children: [] },
        ["3"]: { id: "3", children: ["5"] },
        ["5"]: { id: "5", children: [] },
        ["7"]: { id: "7", children: ["3"] },
      });
      expect(koso.toJSON()).not.toHaveProperty("2");
      expect(koso.toJSON()).not.toHaveProperty("4");
      expect(koso.toJSON()).not.toHaveProperty("6");
    });

    it("delete node 2 from root succeeds and unlinks node 2", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2", children: ["3"] },
        { id: "3", name: "Task 3" },
      ]);

      koso.delete(Node.parse("2").linkage);

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

      koso.delete(Node.parse("1/2").linkage);

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
          children: ["2"],
        },
        { id: "2", name: "Some PR", kind: "github_pr" },
      ]);
      expect(() => koso.delete(Node.parse("github").linkage)).toThrow();
      expect(() =>
        koso.delete(Node.parse("github/github_pr").linkage),
      ).toThrow();
      expect(() =>
        koso.delete(Node.parse("github/github_pr/2").linkage),
      ).toThrow();
    });

    it("delete non-canonical plugin task/container succeeds", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["1", "github", "githubfoo", "4", "5"],
        },
        {
          id: "1",
          name: "Task 1",
          children: ["github", "github_pr", "2"],
        },
        {
          id: "github",
          name: "Github",
          kind: "github",
          children: ["github_pr", "githubfoo"],
        },
        {
          id: "github_pr",
          name: "Github PR",
          kind: "github_pr",
          children: ["2", "githubfoo"],
        },
        {
          id: "2",
          name: "Some PR",
          kind: "github_pr",
        },
        { id: "4", name: "Some Rollup task", kind: "Rollup" },
        { id: "5", name: "Some task", kind: "Task" },
      ]);
      koso.delete(Node.parse("1/github").linkage);
      koso.delete(Node.parse("1/github_pr").linkage);
      koso.delete(Node.parse("1/2").linkage);
      koso.delete(Node.parse("github/githubfoo").linkage);
      koso.delete(Node.parse("github_pr/githubfoo").linkage);
      koso.delete(Node.parse("4").linkage);
      koso.delete(Node.parse("5").linkage);

      expect(koso.toJSON()).toMatchObject({
        ["root"]: { children: ["1", "github", "githubfoo"] },
        ["1"]: { id: "1", children: [] },
        ["github"]: { id: "github", children: ["github_pr"] },
        ["github_pr"]: { id: "github_pr", children: ["2"] },
        ["githubfoo"]: { id: "githubfoo", children: [] },
      });
      expect(koso.toJSON()).not.toHaveProperty("4");
      expect(koso.toJSON()).not.toHaveProperty("5");
    });
  });

  describe("deleteTask", () => {
    it("delete task 2 succeeds and removes task and links", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2" },
      ]);

      koso.deleteTask("2");

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { id: "1", children: [] },
      });
      expect(koso.toJSON()).not.toHaveProperty("2");
    });

    it("delete task 2 succeeds and deletes all orphans", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2", "5"] },
        { id: "1", name: "Task 1", children: ["2", "8"] },
        { id: "2", name: "Task 2", children: ["3", "6", "8"] },
        { id: "3", name: "Task 3", children: ["4", "5"] },
        { id: "4", name: "Task 4" },
        { id: "5", name: "Task 5" },
        { id: "6", name: "Task 6", children: ["7", "4"] },
        { id: "7", name: "Task 7" },
        { id: "8", name: "Task 8", children: ["9"] },
        { id: "9", name: "Task 9" },
      ]);

      koso.deleteTask("2");

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["1", "5"] },
        ["1"]: { id: "1", children: ["8"] },
        ["5"]: { id: "5", children: [] },
        ["8"]: { id: "8", children: ["9"] },
        ["9"]: { id: "9", children: [] },
      });
      expect(koso.toJSON()).not.toHaveProperty("2");
      expect(koso.toJSON()).not.toHaveProperty("4");
      expect(koso.toJSON()).not.toHaveProperty("6");
      expect(koso.toJSON()).not.toHaveProperty("7");
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
          children: ["2"],
        },
        { id: "2", name: "Some PR", kind: "github_pr" },
      ]);
      expect(() => koso.deleteTask("github")).toThrow();
      expect(() => koso.deleteTask("github_pr")).toThrow();
      expect(() => koso.deleteTask("2")).toThrow();
    });
  });

  describe("getTask", () => {
    it("retrieves task 1", () => {
      const task: Task = fullyPopulatedTask();
      init([{ id: "root", name: "Root", children: ["1"] }, task]);

      expect(koso.getTask("1").toJSON()).toStrictEqual<Task>(task);
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
      expect(koso.getParentIds("1")).toEqual(["root"]);
    });

    it("parents of 2 is 1", () => {
      expect(koso.getParentIds("2")).toEqual(["1"]);
    });

    it("parents of 3 are root and 1", () => {
      expect(koso.getParentIds("3")).toEqual(["root", "1"]);
    });

    it("parents of 4 are 3 and 2", () => {
      expect(koso.getParentIds("4")).toEqual(["3", "2"]);
    });

    it("parents of root is empty", () => {
      expect(koso.getParentIds("root")).toEqual([]);
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

    it("returns Blocked for a Blocked task with all children not started", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "3"], kind: "Task", status: "Blocked" },
        { id: "2", status: "Not Started" },
        { id: "3", status: "Not Started" },
      ]);
      expect(koso.getStatus("1")).toBe("Blocked");
    });

    it("returns original status for a not blocked task with all children not started", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        {
          id: "1",
          children: ["2", "3"],
          kind: "Task",
          status: "Not Started",
        },
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
        status: "Not Started",
        childrenStatus: null,
        estimate: null,
        remainingEstimate: null,
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
        status: "Not Started",
        childrenStatus: null,
        estimate: null,
        remainingEstimate: null,
      });
    });

    it("task with no children and Done status has 1 of 1 progress", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: [], statusTime: now, status: "Done" },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 0,
        done: 1,
        total: 1,
        lastStatusTime: now,
        status: "Done",
        childrenStatus: null,
        estimate: null,
        remainingEstimate: null,
      });
    });

    it("task with no children and In Progress status has 0 of 1 progress", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: [], statusTime: now, status: "In Progress" },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 1,
        done: 0,
        total: 1,
        lastStatusTime: now,
        status: "In Progress",
        childrenStatus: null,
        estimate: null,
        remainingEstimate: null,
      });
    });

    it("task with all children done and In Progress status has In Progress status ", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        {
          id: "1",
          children: ["2", "3"],
          statusTime: now,
          kind: "Task",
          status: "In Progress",
        },
        { id: "2", status: "Done", statusTime: 0 },
        { id: "3", status: "Done", statusTime: now },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 1,
        done: 0,
        total: 1,
        lastStatusTime: now,
        status: "In Progress",
        childrenStatus: "Done",
        estimate: null,
        remainingEstimate: null,
      });
    });

    it("task with done status and in progress children has Done status ", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        {
          id: "1",
          children: ["2", "3"],
          statusTime: now,
          kind: "Task",
          status: "Done",
        },
        { id: "2", status: "Not Started", statusTime: 0 },
        { id: "3", status: "In Progress", statusTime: now },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 0,
        done: 1,
        total: 1,
        lastStatusTime: now,
        status: "Done",
        childrenStatus: "In Progress",
        estimate: null,
        remainingEstimate: null,
      });
    });

    it("task with in progress status and in progress children has blocked status ", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        {
          id: "1",
          children: ["2", "3"],
          statusTime: now,
          kind: "Task",
          status: "In Progress",
        },
        { id: "2", status: "Not Started", statusTime: 0 },
        { id: "3", status: "In Progress", statusTime: now },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 1,
        done: 0,
        total: 1,
        lastStatusTime: now,
        status: "In Progress",
        childrenStatus: "In Progress",
        estimate: null,
        remainingEstimate: null,
      });
    });

    it("task with nested not done children has blocked status ", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        {
          id: "1",
          children: ["2"],
          statusTime: now,
          kind: "Task",
          status: "Done",
        },
        { id: "2", status: "Done", children: ["3"], statusTime: 0 },
        { id: "3", status: "In Progress", statusTime: now },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 0,
        done: 1,
        total: 1,
        lastStatusTime: now,
        status: "Done",
        childrenStatus: "In Progress",
        estimate: null,
        remainingEstimate: null,
      });
    });

    it("task no children returns null kind", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        {
          id: "1",
          children: [],
          statusTime: now,
          kind: "Task",
          status: "In Progress",
        },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 1,
        done: 0,
        total: 1,
        lastStatusTime: now,
        status: "In Progress",
        childrenStatus: null,
        estimate: null,
        remainingEstimate: null,
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
        status: "Done",
        childrenStatus: "Done",
        estimate: null,
        remainingEstimate: null,
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
        status: "In Progress",
        childrenStatus: "In Progress",
        estimate: null,
        remainingEstimate: null,
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
        status: "Not Started",
        childrenStatus: "Not Started",
        estimate: null,
        remainingEstimate: null,
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
        status: "In Progress",
        childrenStatus: "In Progress",
        estimate: null,
        remainingEstimate: null,
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
        status: "Done",
        childrenStatus: "Done",
        estimate: null,
        remainingEstimate: null,
      });
    });

    it("parent task with multiple levels of nested children has correct progress", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2"] },
        { id: "2", children: ["3", "4"], estimate: 20 },
        { id: "3", status: "In Progress", statusTime: now, estimate: 2 },
        { id: "4", status: "Done", statusTime: now, estimate: 3 },
      ]);

      expect(koso.getProgress("1")).toEqual({
        inProgress: 1,
        done: 1,
        total: 2,
        lastStatusTime: now,
        status: "In Progress",
        childrenStatus: "In Progress",
        estimate: 5,
        remainingEstimate: 2,
      });
    });

    it("task with no estimate has a null estimate", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: [], estimate: null },
      ]);

      expect(koso.getProgress("1")).toMatchObject({
        estimate: null,
        remainingEstimate: null,
      });
    });

    it("task with estimate", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: [], estimate: 5 },
      ]);

      expect(koso.getProgress("1")).toMatchObject({
        estimate: 5,
      });
    });

    it("rollup task with sparse children estimates has correct estimate", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "3", "4", "5", "6"] },
        { id: "2" },
        { id: "3", estimate: 3 },
        { id: "4", estimate: 5 },
        { id: "5", estimate: 13, status: "Done" },
        { id: "6" },
      ]);

      expect(koso.getProgress("1")).toMatchObject({
        estimate: 21,
        remainingEstimate: 8,
      });
    });

    it("rollup task with duplicate descendants does not double count", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "5"] },
        { id: "2", children: ["3", "4"] },
        { id: "3", estimate: 3 },
        { id: "4", estimate: 5 },
        { id: "5", children: ["2"] },
      ]);

      expect(koso.getProgress("1")).toMatchObject({
        inProgress: 0,
        done: 0,
        total: 2,
        estimate: 8,
        remainingEstimate: 8,
      });
      expect(koso.getProgress("2")).toMatchObject({
        inProgress: 0,
        done: 0,
        total: 2,
        estimate: 8,
        remainingEstimate: 8,
      });
      expect(koso.getProgress("5")).toMatchObject({
        inProgress: 0,
        done: 0,
        total: 2,
        estimate: 8,
        remainingEstimate: 8,
      });
    });

    it("rollup task with duplicate descendants has correct status", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "5"] },
        { id: "2", children: ["3", "4"] },
        { id: "3", estimate: 3, status: "Done" },
        { id: "4", estimate: 5, status: "Done" },
        { id: "5", children: ["2"] },
      ]);

      expect(koso.getProgress("1")).toMatchObject({
        inProgress: 0,
        done: 2,
        total: 2,
        estimate: 8,
        remainingEstimate: 0,
        status: "Done",
      });
      expect(koso.getProgress("2")).toMatchObject({
        inProgress: 0,
        done: 2,
        total: 2,
        estimate: 8,
        remainingEstimate: 0,
        status: "Done",
      });
      expect(koso.getProgress("5")).toMatchObject({
        inProgress: 0,
        done: 2,
        total: 2,
        estimate: 8,
        remainingEstimate: 0,
        status: "Done",
      });
    });

    it("blocked task with duplicate descendants has correct status", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "4"], kind: "Task", status: "Blocked" },
        {
          id: "2",
          children: ["3"],
          kind: "Task",
          status: "Done",
          estimate: 13,
        },
        { id: "3", estimate: 3, status: "In Progress" },
        { id: "4", estimate: 5, children: ["3"] },
      ]);

      expect(koso.getProgress("1")).toMatchObject({
        inProgress: 0,
        done: 0,
        total: 1,
        estimate: null,
        remainingEstimate: null,
        status: "Blocked",
      });
      expect(koso.getProgress("2")).toMatchObject({
        inProgress: 0,
        done: 1,
        total: 1,
        estimate: 13,
        remainingEstimate: 0,
        status: "Done",
      });
      expect(koso.getProgress("4")).toMatchObject({
        inProgress: 1,
        done: 0,
        total: 1,
        estimate: 3,
        remainingEstimate: 3,
        status: "In Progress",
      });
    });

    it("regular task with children and estimate uses own estimate", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", children: ["3", "4", "5"], kind: "Task", estimate: 1 },
        {
          id: "2",
          children: ["3", "4", "5"],
          kind: "Task",
          estimate: 1,
          status: "Done",
        },
        { id: "3", estimate: 3 },
        { id: "4", estimate: 5 },
        { id: "5" },
      ]);

      expect(koso.getProgress("1")).toMatchObject({
        estimate: 1,
        remainingEstimate: 1,
      });
      expect(koso.getProgress("2")).toMatchObject({
        estimate: 1,
        remainingEstimate: 0,
      });
    });

    it("regular task with children and no estimate has null estimate", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        {
          id: "1",
          children: ["2", "3", "4", "5"],
          kind: "Task",
          estimate: null,
        },
        { id: "2" },
        { id: "3", estimate: 3 },
        { id: "4", estimate: 5 },
        { id: "5" },
      ]);

      expect(koso.getProgress("1")).toMatchObject({
        estimate: null,
        remainingEstimate: null,
      });
    });

    it("rollup task with no children is Done", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        {
          id: "1",
          children: [],
          kind: "Rollup",
          estimate: 5,
        },
      ]);

      expect(koso.getProgress("1")).toMatchObject({
        estimate: null,
        remainingEstimate: null,
        status: "Done",
        childrenStatus: null,
      });
    });
  });

  describe("hasDescendant", () => {
    it("returns true when ancestor has descendant", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", children: ["2"] },
        { id: "2", name: "Task 2", children: ["3"] },
        { id: "3", name: "Task 3" },
      ]);

      expect(koso.hasDescendant("1", "3")).toBe(true);
      expect(koso.hasDescendant("root", "3")).toBe(true);
      expect(koso.hasDescendant("1", "2")).toBe(true);
    });

    it("returns false when ancestor does not have descendant", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: ["3"] },
        { id: "2", name: "Task 2", children: ["4"] },
        { id: "3", name: "Task 3" },
        { id: "4", name: "Task 4" },
      ]);

      expect(koso.hasDescendant("1", "4")).toBe(false);
      expect(koso.hasDescendant("2", "3")).toBe(false);
      expect(koso.hasDescendant("3", "1")).toBe(false);
    });

    it("returns true when node is descendant of itself", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1" },
      ]);

      expect(koso.hasDescendant("1", "1")).toBe(true);
      expect(koso.hasDescendant("root", "root")).toBe(true);
    });

    it("handles multiple paths to same descendant", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", children: ["3"] },
        { id: "2", name: "Task 2", children: ["3"] },
        { id: "3", name: "Task 3" },
      ]);

      expect(koso.hasDescendant("root", "3")).toBe(true);
      expect(koso.hasDescendant("1", "3")).toBe(true);
      expect(koso.hasDescendant("2", "3")).toBe(true);
    });

    it("returns false for non-existent nodes", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1" },
      ]);

      expect(koso.hasDescendant("1", "non-existent")).toBe(false);
      expect(koso.hasDescendant("non-existent", "1")).toBe(false);
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

      expect(koso.setTaskStatus("2", "Done", USER)).toBe(true);

      expect(koso.toJSON()).toMatchObject({
        root: { status: null, children: ["1", "2"], assignee: null },
        ["1"]: { status: null, children: [], assignee: null },
        ["2"]: { status: "Done", children: [], assignee: null },
      });
    });

    it("set node status to in-progress succeeds and assigns to user", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
      ]);

      expect(koso.setTaskStatus("2", "In Progress", USER)).toBe(true);

      expect(koso.toJSON()).toMatchObject({
        root: {
          status: null,
          children: ["1", "2"],
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

    it("set node status to in-progress succeeds and assigns to user", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2" },
      ]);

      expect(koso.setTaskStatus("2", "In Progress", OTHER_USER)).toBe(true);

      expect(koso.toJSON()).toMatchObject({
        root: {
          status: null,
          children: ["1", "2"],
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

    it("setting a Done task to Done leaves task unchanged", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3", "t4", "t5"],
        },
        { id: "t1", status: "In Progress" },
        { id: "t4", status: "Done" },
        { id: "t5", status: "In Progress" },
      ]);

      expect(koso.setTaskStatus("t4", "Done", USER)).toBe(false);
      expect(koso.toJSON()).toMatchObject({
        root: {
          children: ["t1", "t2", "t3", "t4", "t5"],
        },
        ["t4"]: {
          status: "Done",
          children: [],
          assignee: null,
        },
      });
    });

    it("set task with complete children to blocked rejected", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        {
          id: "1",
          name: "Task 1",
          kind: "Task",
          children: ["2"],
          status: "In Progress",
        },
        { id: "2", name: "Task 2", status: "Done" },
      ]);

      expect(koso.setTaskStatus("1", "Blocked", USER)).toBe(false);

      expect(koso.toJSON()).toMatchObject({
        root: { status: null, children: ["1", "2"], assignee: null },
        ["1"]: { status: "In Progress", children: ["2"], assignee: null },
        ["2"]: { status: "Done", children: [], assignee: null },
      });
    });

    it("set task with incomplete children to blocked changes status and assignee", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        {
          id: "1",
          name: "Task 1",
          kind: "Task",
          children: ["2"],
          status: "In Progress",
        },
        { id: "2", name: "Task 2", status: "In Progress" },
      ]);

      expect(koso.setTaskStatus("1", "Blocked", USER)).toBe(true);

      expect(koso.toJSON()).toMatchObject({
        root: { status: null, children: ["1", "2"], assignee: null },
        ["1"]: { status: "Blocked", children: ["2"], assignee: "t@koso.app" },
        ["2"]: { status: "In Progress", children: [], assignee: null },
      });
    });
  });

  describe("setKind", () => {
    it("set task 2's kind to Task updates kind and leaves status unchanged", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", status: "In Progress" },
        { id: "2", name: "Task 2", children: ["1"], status: "Not Started" },
      ]);

      expect(koso.setKind("2", "Task")).toBe(true);

      expect(koso.toJSON()).toMatchObject({
        root: { status: null, children: ["1", "2"], assignee: null },
        ["1"]: { status: "In Progress", children: [], assignee: null },
        ["2"]: {
          kind: "Task",
          status: "In Progress",
          children: ["1"],
          assignee: null,
        },
      });
    });

    it("set task kind to Task with complete children updates kind and leaves status done", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1", status: "Done" },
        {
          id: "2",
          name: "Task 2",
          children: ["1", "3"],
          status: "In Progress",
        },
        { id: "3", name: "Task 3", status: "Done" },
      ]);

      expect(koso.setKind("2", "Task")).toBe(true);

      expect(koso.toJSON()).toMatchObject({
        root: { status: null, children: ["1", "2"], assignee: null },
        ["1"]: { status: "Done", children: [], assignee: null },
        ["3"]: { status: "Done", children: [], assignee: null },
        ["2"]: {
          kind: "Task",
          status: "Done",
          children: ["1", "3"],
          assignee: null,
        },
      });
    });

    it("set existing task to makes no changes", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        { id: "2", name: "Task 2", status: "Done", kind: "Task" },
      ]);

      expect(koso.setKind("2", "Task")).toBe(false);

      expect(koso.toJSON()).toMatchObject({
        root: { status: null, children: ["1", "2"], assignee: null },
        ["1"]: { status: null, children: [], assignee: null },
        ["2"]: {
          kind: "Task",
          status: "Done",
          children: [],
          assignee: null,
        },
      });
    });

    it("set task 2's status to rollup succeeds", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        {
          id: "2",
          name: "Task 2",
          kind: "Task",
          status: "Done",
          children: ["1"],
        },
      ]);

      expect(koso.setKind("2", "Rollup")).toBe(true);

      expect(koso.toJSON()).toMatchObject({
        root: { status: null, children: ["1", "2"], assignee: null },
        ["1"]: { status: null, children: [], assignee: null },
        ["2"]: {
          kind: "Rollup",
          status: null,
          children: ["1"],
          assignee: null,
        },
      });
    });

    it("set existing auto-rollup task to rollup store rollup kind", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        {
          id: "2",
          name: "Task 2",
          kind: null,
          status: "Done",
          children: ["1"],
        },
      ]);

      expect(koso.setKind("2", "Rollup")).toBe(true);

      expect(koso.toJSON()).toMatchObject({
        root: { status: null, children: ["1", "2"], assignee: null },
        ["1"]: { status: null, children: [], assignee: null },
        ["2"]: {
          kind: "Rollup",
          status: null,
          children: ["1"],
          assignee: null,
        },
      });
    });

    it("set existing explicit rollup task to rollup does nothing", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        {
          id: "2",
          name: "Task 2",
          kind: "Rollup",
          status: "Done",
          children: ["1"],
        },
      ]);

      expect(koso.setKind("2", "Rollup")).toBe(false);

      expect(koso.toJSON()).toMatchObject({
        root: { status: null, children: ["1", "2"], assignee: null },
        ["1"]: { status: null, children: [], assignee: null },
        ["2"]: {
          kind: "Rollup",
          status: "Done",
          children: ["1"],
          assignee: null,
        },
      });
    });

    it("set existing explicit rollup task to auto changes kind", () => {
      init([
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", name: "Task 1" },
        {
          id: "2",
          name: "Task 2",
          kind: "Rollup",
          status: "Done",
          children: ["1"],
        },
      ]);

      expect(koso.setKind("2", null)).toBe(true);

      expect(koso.toJSON()).toMatchObject({
        root: { status: null, children: ["1", "2"], assignee: null },
        ["1"]: { status: null, children: [], assignee: null },
        ["2"]: {
          kind: null,
          status: null,
          children: ["1"],
        },
      });
    });

    it("set existing explicit task to auto changes kind and leaves status", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        {
          id: "1",
          name: "Task 1",
          kind: "Task",
          status: "Done",
        },
      ]);

      expect(koso.setKind("1", null)).toBe(true);

      expect(koso.toJSON()).toMatchObject({
        root: { status: null, children: ["1"], assignee: null },
        ["1"]: {
          kind: null,
          status: "Done",
        },
      });
    });
  });

  describe("organizeTasks", () => {
    it("sorts peer tasks by status", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3", "t4", "t5", "t6", "t7", "t8"],
        },
        { id: "t1", status: "Not Started" },
        { id: "t2", status: "Not Started" },
        { id: "t3", status: "Done" },
        { id: "t4", status: "Done" },
        { id: "t5", status: "Blocked" },
        { id: "t6", status: "Blocked" },
        { id: "t7", status: "In Progress" },
        { id: "t8", status: "In Progress" },
      ]);

      koso.organizeTasks("root");

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["t7", "t8", "t1", "t2", "t5", "t6", "t3", "t4"] },
      });
    });

    it("sorts peer tasks by archived", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3", "t4", "t5", "t6", "t7", "t8"],
        },
        { id: "t1", status: "Not Started", archived: true },
        { id: "t2", status: "Not Started", archived: false },
        { id: "t3", status: "Done", archived: true },
        { id: "t4", status: "Done", statusTime: Date.now() },
        { id: "t5", status: "Blocked", archived: false },
        { id: "t6", status: "Blocked" },
        { id: "t7", status: "In Progress", archived: true },
        { id: "t8", status: "In Progress" },
      ]);

      koso.organizeTasks("root");

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["t8", "t2", "t5", "t6", "t4", "t7", "t1", "t3"] },
      });
    });

    it("archives and sorts by archived state", () => {
      const now = Date.now();
      const oneDay = 24 * 60 * 60 * 1000;
      const ancient = now - 15 * oneDay;
      const recent = now - 2 * oneDay;
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t2", "t3", "t4", "t5", "t6", "t7", "t10", "t13"],
        },
        { id: "t1", status: "In Progress", statusTime: ancient },
        { id: "t2", status: "In Progress", statusTime: recent },
        { id: "t3", status: "Done", statusTime: ancient },
        { id: "t4", status: "Done", statusTime: ancient },
        { id: "t5", status: "Done", statusTime: recent },
        { id: "t6", status: "Done", statusTime: null },
        // Done, recent rollup
        { id: "t7", statusTime: null, children: ["t8", "t9"] },
        { id: "t8", status: "Done", statusTime: ancient },
        { id: "t9", status: "Done", statusTime: recent },
        // Done, ancient rollup
        { id: "t10", statusTime: null, children: ["t11", "t12"] },
        { id: "t11", status: "Done", statusTime: ancient },
        { id: "t12", status: "Done", statusTime: ancient },
        // In Progress, recent rollup
        { id: "t13", statusTime: null, children: ["t14"] },
        { id: "t14", status: "In Progress", statusTime: ancient },
      ]);

      koso.organizeTasks("root");

      expect(koso.toJSON()).toMatchObject({
        root: {
          children: ["t1", "t2", "t13", "t5", "t7", "t3", "t4", "t6", "t10"],
        },
        t1: { archived: null },
        t2: { archived: null },
        t3: { archived: true },
        t4: { archived: true },
        t5: { archived: null },
        t6: { archived: true },
        t7: { archived: null },
        t10: { archived: true },
        t13: { archived: null },
      });
    });

    it("sort is stable", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t7", "t8", "t1", "t2", "t5", "t6", "t3", "t4"],
        },
        { id: "t1", status: "Not Started" },
        { id: "t2", status: "Not Started" },
        { id: "t3", status: "Done" },
        { id: "t4", status: "Done" },
        { id: "t5", status: "Blocked" },
        { id: "t6", status: "Blocked" },
        { id: "t7", status: "In Progress" },
        { id: "t8", status: "In Progress" },
      ]);

      koso.organizeTasks("root");

      expect(koso.toJSON()).toMatchObject({
        root: { children: ["t7", "t8", "t1", "t2", "t5", "t6", "t3", "t4"] },
      });

      koso.organizeTasks("root");

      expect(koso.toJSON()).toMatchObject({
        root: {
          children: ["t7", "t8", "t1", "t2", "t5", "t6", "t3", "t4"],
        },
      });
    });

    it("sorts rollup tasks by rollup status", () => {
      init([
        {
          id: "root",
          name: "Root",
          children: ["t1", "t4", "t3", "t2"],
        },
        { id: "t1", status: "In Progress" },
        { id: "t2", status: "Not Started", children: ["t5", "t6"] },
        { id: "t3", status: "In Progress", children: ["t7", "t8"] },
        { id: "t4", status: "In Progress", children: ["t9", "t10"] },
        { id: "t5", status: "In Progress" },
        { id: "t6", status: "Not Started" },
        { id: "t7", status: "Not Started" },
        { id: "t8" },
        { id: "t9", status: "Done" },
        { id: "t10", status: "Done" },
      ]);

      koso.organizeTasks("root");

      expect(koso.toJSON()).toMatchObject({
        root: {
          children: ["t1", "t2", "t3", "t4"],
        },
      });
    });
  });

  describe("setTaskArchived", () => {
    it("set task 1 as archived", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1" },
      ]);

      koso.setTaskArchived("1", true);

      expect(koso.toJSON()).toMatchObject({
        ["1"]: {
          archived: true,
        },
      });
    });
  });

  it("set task 1 as unarchived", () => {
    init([
      { id: "root", name: "Root", children: ["1"] },
      { id: "1", name: "Task 1", archived: true },
    ]);

    koso.setTaskArchived("1", false);

    expect(koso.toJSON()).toMatchObject({
      ["1"]: {
        archived: false,
      },
    });
  });
});
