import { defaultTask } from "$lib/yproxy";
import { Set } from "immutable";
import { uuidv4 } from "lib0/random.js";
import { beforeEach, describe, expect, it } from "vitest";
import * as Y from "yjs";
import {
  buildTask,
  EMPTY_SYNC_RESPONSE,
  type TaskBuilder,
} from "../../../tests/utils";
import { InboxContext } from "./inbox-context.svelte";
import { Koso } from "./koso.svelte";

describe("InboxContext tests", () => {
  const YOU = "you@koso.app";
  const OTHER = "other@koso.app";
  let koso: Koso;
  let inboxCtx: InboxContext;

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
      inboxCtx = new InboxContext(
        { email: YOU, name: "You", picture: "" },
        koso,
      );
      koso.setSendAndSync(() => {});
      koso.receive(EMPTY_SYNC_RESPONSE);
    });
    context.onTestFinished(() => cleanup());
  });

  describe("actionable action items", () => {
    it("no items when the graph is empty", () => {
      init([{ id: "root", name: "Root", children: [] }]);

      expect(inboxCtx.actionItems).toMatchObject([]);
    });

    it("shows task when it is assigned to you", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", assignee: YOU },
      ]);

      expect(inboxCtx.actionItems).toMatchObject([
        { task: { id: "1" }, reasons: [{ name: "Actionable" }] },
      ]);
    });

    it("no items when the task is unassigned", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1" },
      ]);

      expect(inboxCtx.actionItems).toMatchObject([]);
    });

    it("no items when the task is assigned to someone else", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Task 1", assignee: OTHER },
      ]);

      expect(inboxCtx.actionItems).toMatchObject([]);
    });
  });

  describe("owner of parent action items", () => {
    it("shows unassigned child when parent rollup is assigned to you", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Parent", assignee: YOU, children: ["2"] },
        { id: "2", name: "Child", assignee: null },
      ]);

      expect(inboxCtx.actionItems).toMatchObject([
        {
          task: { id: "2" },
          reasons: [{ name: "ParentOwner", parents: [{ id: "1" }] }],
        },
      ]);
    });

    it("no items when parent rollup is assigned to someone else", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Parent", assignee: OTHER, children: ["2"] },
        { id: "2", name: "Child", assignee: null },
      ]);

      expect(inboxCtx.actionItems).toMatchObject([]);
    });

    it("no items when child is already assigned", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Parent", assignee: YOU, children: ["2"] },
        { id: "2", name: "Child", assignee: OTHER },
      ]);

      expect(inboxCtx.actionItems).toMatchObject([]);
    });

    it("no items when child is Ready", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Parent", assignee: YOU, children: ["2"] },
        { id: "2", name: "Child", assignee: null, status: "Ready" },
      ]);

      expect(inboxCtx.actionItems).toMatchObject([]);
    });

    it("no items when child is complete", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Parent", assignee: YOU, children: ["2"] },
        { id: "2", name: "Child", assignee: null, status: "Done" },
      ]);

      expect(inboxCtx.actionItems).toMatchObject([]);
    });

    it("shows multiple children when parent rollup is assigned to you", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Parent", assignee: YOU, children: ["2", "3"] },
        { id: "2", name: "Child 1", assignee: null },
        { id: "3", name: "Child 2", assignee: null },
      ]);

      expect(inboxCtx.actionItems).toMatchObject([
        {
          task: { id: "2" },
          reasons: [{ name: "ParentOwner", parents: [{ id: "1" }] }],
        },
        {
          task: { id: "3" },
          reasons: [{ name: "ParentOwner", parents: [{ id: "1" }] }],
        },
      ]);
    });
  });

  describe("tasks needing estimation", () => {
    const TWO_WEEKS = Date.now() + 14 * 24 * 60 * 60 * 1000;

    it("shows task when it needs estimation for current iteration", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Iter 1", children: ["2"], deadline: TWO_WEEKS },
        { id: "2", name: "Task 2", assignee: YOU, estimate: null },
      ]);

      expect(inboxCtx.actionItems).toMatchObject([
        {
          task: { id: "2" },
          reasons: [
            {
              name: "NeedsEstimate",
              iteration: { id: "1" },
            },
            { name: "Actionable" },
          ],
        },
      ]);
    });

    it("no items when task already has an estimate", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Iter 1", children: ["2"], deadline: TWO_WEEKS },
        { id: "2", name: "Task 2", assignee: YOU, estimate: 3 },
      ]);

      expect(inboxCtx.actionItems).toMatchObject([
        {
          task: { id: "2" },
          reasons: [{ name: "Actionable" }],
        },
      ]);
    });

    it("no items when task is not assigned to you", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Iter 1", children: ["2"], deadline: TWO_WEEKS },
        { id: "2", name: "Task 2", assignee: OTHER },
      ]);

      expect(inboxCtx.actionItems).toMatchObject([]);
    });

    it("no items when task is complete", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Iter 1", children: ["2"], deadline: TWO_WEEKS },
        { id: "2", name: "Task 2", assignee: YOU, status: "Done" },
      ]);

      expect(inboxCtx.actionItems).toMatchObject([]);
    });

    it("shows unassigned task in current iteration needing estimate", () => {
      init([
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", name: "Iter 1", children: ["2"], deadline: TWO_WEEKS },
        { id: "2", name: "Task 2", assignee: null, estimate: null },
      ]);

      expect(inboxCtx.actionItems).toMatchObject([
        {
          task: { id: "2" },
          reasons: [
            {
              name: "NeedsEstimate",
              actions: { estimate: expect.any(Number), assign: 1 },
              iteration: { id: "1" },
            },
          ],
        },
      ]);
    });
  });
});
