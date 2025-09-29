<script lang="ts">
  import { goto, replaceState } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { page } from "$app/state";
  import { AnthropicStream } from "$lib/anthropic.svelte";
  import { headers } from "$lib/api";
  import { getAuthContext } from "$lib/auth.svelte";
  import { getRegistryContext } from "$lib/components/ui/command-palette";
  import {
    ActionIds,
    Categories,
  } from "$lib/components/ui/command-palette/command-palette.svelte";
  import { KosoLogo } from "$lib/components/ui/koso-logo";
  import { getPrefsContext } from "$lib/components/ui/prefs";
  import { toast } from "$lib/components/ui/sonner";
  import { GeminiStream } from "$lib/gemini.svelte";
  import { CANCEL, INSERT_CHILD_NODE, INSERT_NODE } from "$lib/shortcuts";
  import type { User } from "$lib/users";
  import {
    Archive,
    ArchiveRestore,
    Cable,
    Check,
    ChevronsDownUp,
    ChevronsUpDown,
    CircleGauge,
    CircleX,
    Clipboard,
    Eye,
    EyeOff,
    Hash,
    IndentDecrease,
    IndentIncrease,
    ListEnd,
    ListPlus,
    ListStart,
    ListTree,
    MoveDown,
    MoveUp,
    OctagonX,
    Pencil,
    Plus,
    Redo,
    Search,
    Share,
    SkipBack,
    SkipForward,
    Sparkles,
    SquarePen,
    StepBack,
    StepForward,
    Trash,
    Undo,
    UserRoundPlus,
    Wrench,
  } from "@lucide/svelte";
  import { Action, Button, Fab, getDialoguerContext, Shortcut } from "kosui";
  import { onMount, tick } from "svelte";
  import { flip } from "svelte/animate";
  import DagRow from "./dag-row.svelte";
  import { getPlanningContext, Node } from "./planning-context.svelte";
  import { getProjectContext } from "./project-context.svelte";
  import SearchPanel from "./search-panel.svelte";
  import TaskEstimateHeading from "./task-estimate-heading.svelte";

  type Props = {
    users: User[];
    hideFab?: boolean;
  };
  const { users, hideFab = false }: Props = $props();

  const rows: { [key: string]: DagRow } = {};

  const command = getRegistryContext();
  const prefs = getPrefsContext();
  const projectCtx = getProjectContext();
  const planningCtx = getPlanningContext();
  const { koso } = planningCtx;
  const auth = getAuthContext();
  const dialog = getDialoguerContext();

  function getRow(node: Node) {
    const maybeRow = rows[node.id];
    if (!maybeRow) {
      throw new Error(`Row doesn't exist for ${node}`);
    }
    return maybeRow;
  }

  let searchPaletteOpen: boolean = $state(false);
  function showSearchPalette() {
    searchPaletteOpen = true;
  }

  function insertAndEdit(parent: Node, offset: number, user: User) {
    const taskId = koso.insertTask({
      parent: parent.name,
      offset,
      reporter: user.email,
    });
    const node = parent.child(taskId);
    planningCtx.selected = node;

    // The newly inserted node's row won't yet have been inserted into
    // the dom and thus onMount will not have been called to register
    // row callbacks.
    // Delay interacting with the row registry to start editing.
    tick().then(() => getRow(node).edit(true));
  }

  function insert() {
    if (planningCtx.selected) {
      insertAndEdit(
        planningCtx.selected.parent,
        planningCtx.getOffset(planningCtx.selected) + 1,
        auth.user,
      );
    } else {
      insertAndEdit(planningCtx.root, 0, auth.user);
    }
  }

  function insertAbove() {
    if (!planningCtx.selected) return;
    insertAndEdit(
      planningCtx.selected.parent,
      planningCtx.getOffset(planningCtx.selected),
      auth.user,
    );
  }

  function insertChild() {
    if (!planningCtx.selected) return;
    planningCtx.expand(planningCtx.selected);
    insertAndEdit(planningCtx.selected, 0, auth.user);
  }

  function insertChildAbove() {
    if (!planningCtx.selected) return;

    const previousPeer = planningCtx.getPrevPeer(planningCtx.selected);
    if (!previousPeer) return;

    planningCtx.expand(previousPeer);
    const lastIndex = koso.getChildCount(previousPeer.name);
    insertAndEdit(previousPeer, lastIndex, auth.user);
  }

  function toggleStatus() {
    if (!planningCtx.selected) return;
    koso.toggleStatus(planningCtx.selected.name, auth.user);
  }

  function remove() {
    if (!planningCtx.selected) return;
    const toDelete = planningCtx.selected;
    const toDeleteIndex = planningCtx.nodes.indexOf(toDelete);

    koso.delete(toDelete.linkage);

    // Select the next (or previous) node following deletion.
    if (planningCtx.nodes.size < 2 || toDeleteIndex <= 0) {
      planningCtx.selected = null;
    } else {
      planningCtx.selected =
        planningCtx.nodes.get(
          Math.min(toDeleteIndex, planningCtx.nodes.size - 1),
        ) || null;
    }
  }

  function archive() {
    if (!planningCtx.selected) return;
    koso.setTaskArchived(planningCtx.selected.name, true);
  }
  function unarchive() {
    if (!planningCtx.selected) return;
    koso.setTaskArchived(planningCtx.selected.name, false);
  }
  function edit() {
    if (!planningCtx.selected) return;
    getRow(planningCtx.selected).edit(true);
  }

  function unselect() {
    planningCtx.selected = null;
  }

  function moveUp() {
    if (!planningCtx.selected) return;
    planningCtx.moveNodeUp(planningCtx.selected);
  }

  function moveDown() {
    if (!planningCtx.selected) return;
    planningCtx.moveNodeDown(planningCtx.selected);
  }

  function moveStart() {
    if (!planningCtx.selected) return;
    planningCtx.moveNodeUpBoundary(planningCtx.selected);
  }

  function moveEnd() {
    if (!planningCtx.selected) return;
    planningCtx.moveNodeDownBoundary(planningCtx.selected);
  }

  function indent() {
    if (!planningCtx.selected) return;
    planningCtx.indentNode(planningCtx.selected);
  }

  function undent() {
    if (!planningCtx.selected) return;
    planningCtx.undentNode(planningCtx.selected);
  }

  function expand() {
    if (!planningCtx.selected) return;
    planningCtx.expand(planningCtx.selected);
  }

  function collapse() {
    if (!planningCtx.selected) return;
    planningCtx.collapse(planningCtx.selected);
  }

  function showArchivedTasks() {
    planningCtx.showArchived = true;
  }

  function hideArchivedTasks() {
    planningCtx.showArchived = false;
  }

  function selectNext() {
    if (planningCtx.nodes.size > 1) {
      if (planningCtx.selected) {
        const selectedIndex = planningCtx.nodes.indexOf(planningCtx.selected);
        if (selectedIndex <= 0) {
          planningCtx.selected = null;
        } else {
          const index = Math.min(selectedIndex + 1, planningCtx.nodes.size - 1);
          planningCtx.selected = planningCtx.nodes.get(index, null);
        }
      } else {
        planningCtx.selected = planningCtx.nodes.get(1, null);
      }
    }
  }

  function selectPrev() {
    if (planningCtx.nodes.size > 1) {
      if (planningCtx.selected) {
        const selectedIndex = planningCtx.nodes.indexOf(planningCtx.selected);
        if (selectedIndex <= 0) {
          planningCtx.selected = null;
        } else {
          const index = Math.max(selectedIndex - 1, 1);
          planningCtx.selected = planningCtx.nodes.get(index, null);
        }
      } else {
        planningCtx.selected = planningCtx.nodes.get(
          planningCtx.nodes.size - 1,
          null,
        );
      }
    }
  }

  function selectNextLink() {
    if (planningCtx.selected) {
      const next = planningCtx.getNextLink(planningCtx.selected);
      if (next) {
        planningCtx.selected = next;
      }
    }
  }

  function selectPrevLink() {
    if (planningCtx.selected) {
      const prev = planningCtx.getPrevLink(planningCtx.selected);
      if (prev) {
        planningCtx.selected = prev;
      }
    }
  }

  function undo() {
    planningCtx.undo();
  }

  function redo() {
    planningCtx.redo();
  }

  function linkTask() {
    if (!planningCtx.selected) return;
    getRow(planningCtx.selected).linkPanel(true, "link");
  }

  function blockTask() {
    if (!planningCtx.selected) return;
    getRow(planningCtx.selected).linkPanel(true, "block");
  }

  function organizeTasks() {
    if (!planningCtx.selected) return;
    koso.organizeTasks(planningCtx.selected.parent.name);
  }

  function copyTaskId() {
    if (!planningCtx.selected) return;
    const taskId = planningCtx.selected.name;
    navigator.clipboard.writeText(koso.getGitCommitId(taskId));
  }

  function copyTaskLink() {
    if (!planningCtx.selected) return;
    navigator.clipboard.writeText(
      koso.getTaskPermalink(planningCtx.selected.name).toString(),
    );
  }

  async function breakDownTask() {
    if (!planningCtx.selected) return;
    planningCtx.expand(planningCtx.selected);
    const projectId = koso.projectId;
    const taskId = planningCtx.selected.name;

    const summary = new AnthropicStream().onLine((line) => {
      koso.insertTask({
        name: line,
        parent: taskId,
        offset: koso.getChildCount(taskId),
        reporter: auth.user.email,
      });
    });

    const response = summary.fetch(
      `/api/anthropic/breakdown?projectId=${projectId}&taskId=${taskId}&model=claude-sonnet-4-20250514`,
      {
        method: "GET",
        headers: headers(auth),
      },
    );

    toast.promise(response, {
      loading: "Koso Agent is breaking down the task...",
      success: "Task break down complete!",
      error: "Koso Agent encountered an error while breaking down the task.",
    });

    return await response;
  }

  async function generateDesignDoc() {
    if (!planningCtx.selected) return;
    prefs.detailPanel = "view";
    const projectId = koso.projectId;

    const task = koso.getTask(planningCtx.selected.name);

    const match = task.name.match(/github.com\/([\w.-]+)\/([\w.-]+)/);
    if (match === null) {
      toast.error(
        "Add a link to the GitHub repo in the task's name that you would like the Koso Agent to summarize.",
      );
      return;
    }

    const owner = match[1];
    const repo = match[2];

    if (task.desc !== null) {
      const result = await dialog.confirm({
        title: "Overwrite existing description?",
        message:
          "The task has an existing description. Generating a new design doc will overwrite it.",
        acceptText: "Overwrite",
        icon: Trash,
      });

      if (!result) return;
    }

    task.delDesc();
    const desc = task.newDesc();

    const response = new GeminiStream()
      .onLine((token) => {
        desc.insert(desc.length, token);
      })
      .fetch(
        `/api/gemini/context?projectId=${projectId}&owner=${owner}&repo=${repo}`,
        {
          method: "GET",
          headers: headers(auth),
        },
      );

    toast.promise(response, {
      loading: "Koso Agent is generating the design doc...",
      success: "Done!",
      error: "Koso Agent encountered an error while generating the design doc.",
    });

    return await response;
  }

  const insertAction: Action = new Action({
    id: ActionIds.Insert,
    callback: insert,
    category: Categories.Task,
    name: "New Task",
    description: "Add or insert a new task",
    icon: ListPlus,
    shortcut: INSERT_NODE,
    enabled: () =>
      !planningCtx.selected || koso.canInsert(planningCtx.selected.parent.name),
  });

  const actions: Action[] = [
    // Select
    new Action({
      id: ActionIds.Next,
      callback: selectNext,
      category: Categories.Select,
      name: "Next Task",
      description: "Select next task",
      icon: StepForward,
      shortcut: new Shortcut({ key: "ArrowDown" }),
    }),
    new Action({
      id: ActionIds.Previous,
      callback: selectPrev,
      category: Categories.Select,
      name: "Previous Task",
      description: "Select previous task",
      icon: StepBack,
      shortcut: new Shortcut({ key: "ArrowUp" }),
    }),
    new Action({
      id: ActionIds.Clear,
      callback: unselect,
      category: Categories.Select,
      name: "Deselect Task",
      description: "Clear the current selection",
      icon: CircleX,
      shortcut: CANCEL,
    }),
    new Action({
      id: ActionIds.NextLink,
      callback: selectNextLink,
      category: Categories.Select,
      name: "Jump to Next Task Link",
      description: "Select next link to current task",
      icon: SkipForward,
      enabled: () => !!planningCtx.selected,
      shortcut: new Shortcut({ key: "ArrowDown", meta: true }),
    }),
    new Action({
      id: ActionIds.PreviousLink,
      callback: selectPrevLink,
      category: Categories.Select,
      name: "Jump to Previous Task Link",
      description: "Select previous link to current task",
      icon: SkipBack,
      enabled: () => !!planningCtx.selected,
      shortcut: new Shortcut({ key: "ArrowUp", meta: true }),
    }),
    new Action({
      id: ActionIds.Search,
      callback: showSearchPalette,
      category: Categories.Select,
      name: "Search for Task...",
      description: "Show the task search palette",
      icon: Search,
      shortcut: new Shortcut({ key: "p", meta: true }),
    }),

    insertAction,
    new Action({
      id: ActionIds.InsertAbove,
      callback: insertAbove,
      category: Categories.Task,
      name: "New Above",
      description: "Insert a new task above",
      icon: ListPlus,
      shortcut: new Shortcut({ key: "Enter", meta: true, shift: true }),
      enabled: () =>
        !!planningCtx.selected &&
        koso.canInsert(planningCtx.selected.parent.name),
    }),
    new Action({
      id: ActionIds.InsertSubtask,
      callback: insertChild,
      category: Categories.Task,
      name: "New Child",
      description: "Insert a new subtask as a child of the current task",
      icon: ListTree,
      enabled: () =>
        !!planningCtx.selected && koso.canInsert(planningCtx.selected.name),
      shortcut: INSERT_CHILD_NODE,
    }),
    new Action({
      id: ActionIds.InsertSubtaskAbove,
      callback: insertChildAbove,
      category: Categories.Task,
      name: "New Child Above",
      description: "Insert a new subtask as a child of the previous task",
      icon: ListTree,
      enabled: () => {
        if (!planningCtx.selected) {
          return false;
        }
        const prevPeer = planningCtx.getPrevPeer(planningCtx.selected);
        return !!prevPeer && koso.canInsert(prevPeer.name);
      },
      shortcut: new Shortcut({
        key: "Enter",
        alt: true,
        meta: true,
        shift: true,
      }),
    }),
    new Action({
      id: ActionIds.MoveUp,
      callback: moveUp,
      category: Categories.Task,
      name: "Move Task Up",
      description: "Move the current task up",
      icon: MoveUp,
      enabled: () => !!planningCtx.selected,
      shortcut: new Shortcut({ key: "ArrowUp", alt: true }),
    }),
    new Action({
      id: ActionIds.MoveDown,
      callback: moveDown,
      category: Categories.Task,
      name: "Move Task Down",
      description: "Move the current task down",
      icon: MoveDown,
      enabled: () => !!planningCtx.selected,
      shortcut: new Shortcut({ key: "ArrowDown", alt: true }),
    }),
    new Action({
      id: ActionIds.MoveToStart,
      callback: moveStart,
      category: Categories.Task,
      name: "Move Task to Start",
      description: "Move the current task to the top of its group",
      icon: ListStart,
      enabled: () => !!planningCtx.selected,
      shortcut: new Shortcut({ key: "ArrowUp", alt: true, shift: true }),
    }),
    new Action({
      id: ActionIds.MoveToEnd,
      callback: moveEnd,
      category: Categories.Task,
      name: "Move Task to End",
      description: "Move the current task to the bottom of its group",
      icon: ListEnd,
      enabled: () => !!planningCtx.selected,
      shortcut: new Shortcut({ key: "ArrowDown", alt: true, shift: true }),
    }),
    new Action({
      id: ActionIds.Indent,
      callback: indent,
      category: Categories.Task,
      name: "Indent",
      description: "Make the current task a child of its peer",
      icon: IndentIncrease,
      enabled: () =>
        !!planningCtx.selected &&
        planningCtx.canIndentNode(planningCtx.selected),
      shortcut: new Shortcut({ key: "ArrowRight", alt: true }),
    }),
    new Action({
      id: ActionIds.Undent,
      callback: undent,
      category: Categories.Task,
      name: "Unindent",
      description: "Make the current task a peer of its parent",
      icon: IndentDecrease,
      enabled: () =>
        !!planningCtx.selected &&
        planningCtx.canUndentNode(planningCtx.selected),
      shortcut: new Shortcut({ key: "ArrowLeft", alt: true }),
    }),

    new Action({
      id: ActionIds.Expand,
      callback: expand,
      category: Categories.View,
      name: "Expand Selected Task",
      description: "Expand the current task",
      icon: ChevronsUpDown,
      enabled: () =>
        !!planningCtx.selected && planningCtx.canExpand(planningCtx.selected),
      shortcut: new Shortcut({ key: "ArrowRight" }),
    }),
    new Action({
      id: ActionIds.Collapse,
      callback: collapse,
      category: Categories.View,
      name: "Collapse Selected Task",
      description: "Collapse the current task",
      icon: ChevronsDownUp,
      enabled: () =>
        !!planningCtx.selected && planningCtx.canCollapse(planningCtx.selected),
      shortcut: new Shortcut({ key: "ArrowLeft" }),
    }),
    new Action({
      id: ActionIds.ExpandAll,
      callback: () => planningCtx.expandAll(),
      category: Categories.View,
      name: "Expand All",
      description: "Expand all tasks",
      icon: ChevronsUpDown,
    }),
    new Action({
      id: ActionIds.CollapseAll,
      callback: () => planningCtx.collapseAll(),
      category: Categories.View,
      name: "Collapse All",
      description: "Collapse all tasks",
      icon: ChevronsDownUp,
    }),
    new Action({
      id: ActionIds.HideArchivedTasks,
      callback: hideArchivedTasks,
      category: Categories.View,
      name: "Hide Archived Tasks",
      description: "Hide tasks that have been marked archived",
      icon: EyeOff,
      enabled: () => planningCtx.showArchived,
    }),
    new Action({
      id: ActionIds.ShowArchivedTasks,
      callback: showArchivedTasks,
      category: Categories.View,
      name: "Show Archived Tasks",
      description: "Show tasks that have been archived",
      icon: Eye,
      enabled: () => !planningCtx.showArchived,
    }),

    new Action({
      id: ActionIds.Undo,
      callback: undo,
      category: Categories.Edit,
      name: "Undo",
      icon: Undo,
      shortcut: new Shortcut({ key: "z", meta: true }),
    }),
    new Action({
      id: ActionIds.Redo,
      callback: redo,
      category: Categories.Edit,
      name: "Redo",
      icon: Redo,
      shortcut: new Shortcut({ key: "z", meta: true, shift: true }),
    }),
    new Action({
      id: ActionIds.Edit,
      callback: edit,
      category: Categories.Edit,
      name: "Edit Task Name",
      description: "Edit the current task",
      icon: Pencil,
      shortcut: new Shortcut({ key: "Enter" }),
      enabled: () =>
        !!planningCtx.selected && koso.isEditable(planningCtx.selected.name),
    }),
    new Action({
      id: ActionIds.ToggleTaskStatus,
      callback: toggleStatus,
      category: Categories.Edit,
      name: "Toggle Task Status",
      description: "Toggle the task status to In Progress or Done",
      icon: Check,
      shortcut: new Shortcut({ key: " " }),
      enabled: () =>
        !!planningCtx.selected && koso.isEditable(planningCtx.selected.name),
    }),
    new Action({
      id: ActionIds.Delete,
      callback: remove,
      category: Categories.Edit,
      name: "Delete Task",
      description: "Delete the current task",
      icon: Trash,
      enabled: () =>
        !!planningCtx.selected && koso.canDelete(planningCtx.selected.linkage),
      shortcut: new Shortcut({ key: "Delete" }),
    }),
    new Action({
      id: ActionIds.CopyTaskInfo,
      callback: copyTaskId,
      category: Categories.Edit,
      name: "Copy Task ID",
      description: "Copy task ID to the clipboard",
      icon: Clipboard,
      enabled: () => !!planningCtx.selected,
    }),
    new Action({
      id: ActionIds.CopyTaskLink,
      callback: copyTaskLink,
      category: Categories.Edit,
      name: "Copy Task Permalink",
      description: "Share task by copying permalink to the clipboard",
      icon: Share,
      shortcut: new Shortcut({ key: "c", meta: true, shift: true }),
      enabled: () => !!planningCtx.selected,
    }),
    new Action({
      id: ActionIds.Archive,
      callback: archive,
      category: Categories.Edit,
      name: "Archive Task",
      description: "Archive the current task",
      icon: Archive,
      shortcut: new Shortcut({ key: "e" }),
      enabled: () =>
        !!planningCtx.selected &&
        !koso.getTask(planningCtx.selected.name).archived,
    }),
    new Action({
      id: ActionIds.Unarchive,
      callback: unarchive,
      category: Categories.Edit,
      name: "Unarchive Task",
      description: "Unarchive the current task",
      icon: ArchiveRestore,
      shortcut: new Shortcut({ key: "e", meta: true }),
      enabled: () =>
        !!planningCtx.selected &&
        !!koso.getTask(planningCtx.selected.name).archived,
    }),

    new Action({
      id: ActionIds.Link,
      callback: linkTask,
      category: Categories.Graph,
      name: "Link Task To...",
      description: "Link current task to another task",
      icon: Cable,
      enabled: () => !!planningCtx.selected,
      shortcut: new Shortcut({ key: "/", meta: true }),
    }),
    new Action({
      id: ActionIds.Block,
      callback: blockTask,
      category: Categories.Graph,
      name: "Block Task On...",
      description: "Block current task to another task",
      icon: OctagonX,
      enabled: () => !!planningCtx.selected,
    }),
    new Action({
      id: ActionIds.Organize,
      callback: organizeTasks,
      category: Categories.Graph,
      name: "Organize Tasks",
      description: "Organize the current task and its peers",
      icon: Wrench,
      enabled: () => !!planningCtx.selected,
    }),

    new Action({
      id: ActionIds.DashView,
      callback: () =>
        goto(
          resolve(
            `/projects/${projectCtx.id}/dash/${planningCtx.selected?.name}`,
          ),
        ),
      category: Categories.Navigation,
      name: "Dashboard",
      description: "Navigate to Project Dashboard view",
      icon: CircleGauge,
      enabled: () =>
        !!planningCtx.selected &&
        koso.getTask(planningCtx.selected.name).isRollup() &&
        koso.getTask(planningCtx.selected.name).deadline !== null,
    }),

    new Action({
      id: ActionIds.BreakDown,
      callback: breakDownTask,
      category: Categories.Agent,
      name: "Break Down Subtasks",
      description: "Break down the current task using Koso Agent",
      icon: Sparkles,
      enabled: () =>
        !!planningCtx.selected &&
        koso.getTask(planningCtx.selected.name).isTask() &&
        koso.getStatus(planningCtx.selected.name) !== "Done" &&
        koso.getChildCount(planningCtx.selected.name) === 0,
    }),
    new Action({
      id: ActionIds.GenerateDesignDoc,
      callback: generateDesignDoc,
      category: Categories.Agent,
      name: "Generate Design Doc",
      description: "Generate the design doc GitHub repo using Koso Agent",
      icon: Sparkles,
      enabled: () => !!planningCtx.selected,
    }),
  ];

  onMount(async () => {
    const url = page.url;
    const taskId = url.searchParams.get("taskId");
    if (taskId) {
      await koso.synced;
      url.searchParams.delete("taskId");
      // ResolvedPathName not yet supported.
      // eslint-disable-next-line svelte/no-navigation-without-resolve
      replaceState(url, {});
      // The task may not exist locally, yet. It
      // might come from the server, so wait for that.
      if (planningCtx.koso.getTaskIndex(taskId) < 0) {
        console.debug(
          `Waiting for server sync before selecting task ${taskId}`,
        );
        await koso.serverSynced;
        await tick();

        if (planningCtx.koso.getTaskIndex(taskId) < 0) {
          console.warn(
            `Cannot select ${taskId} after server sync. It doesn't exist`,
          );
          toast.warning(`Task not found. It may have been deleted.`);
          return;
        }
      }

      planningCtx.select(taskId);
    }
  });

  onMount(() => {
    // Clear selection on destroy to avoid persisting awareness on navigation.
    return () => {
      planningCtx.selected = null;
    };
  });

  onMount(() => {
    return command.register(...actions);
  });

  // This effect selects a new node when the
  // selected node no longer exists. For example, when
  // the user marks a task as done or blocked in the inbox
  // or a different user deletes the user's currently selected node.
  $effect(() => {
    const selected = planningCtx.selectedRaw;
    const node = selected.node;
    const index = selected.index;

    if (!node) {
      return;
    }

    const currentIndex = planningCtx.nodes.indexOf(node);
    if (currentIndex !== -1) {
      // The node still exists. Make sure the stashed index still matches.
      if (!index || index !== currentIndex) {
        console.debug(
          `Refreshing selected index for node ${node.id} at prior index ${index}`,
        );
        planningCtx.selected = node;
      }
      return;
    }

    // The selected node no longer exists. Select the
    // node at the same index or the one at the end of the list.
    // The first node is not selectable.
    if (planningCtx.nodes.size > 1) {
      console.debug(`Node ${node.id} no longer exists. Selecting new node.`);
      planningCtx.selected = planningCtx.nodes.get(
        Math.min(index || -1, planningCtx.nodes.size - 1),
        null,
      );
    } else {
      console.debug(`Node ${node.id} no longer exists. Clearing selection.`);
      planningCtx.selected = null;
    }
  });
</script>

<SearchPanel bind:open={searchPaletteOpen} />

{#await koso.synced then}
  {#if planningCtx.nodes.size > 1}
    <div class="relative h-full">
      <div class="flex h-full flex-col gap-1.5">
        <!-- Add a z-0 to fix a bug in Safari where rows disappear when collapsing
                 and expanding tasks -->
        <table
          class="dag-table z-0 w-full border-separate border-spacing-0 rounded-md border"
        >
          <thead class="text-left text-xs font-bold uppercase">
            <tr>
              <th class="relative m-0 w-0 p-0"></th>
              <th class="w-32 p-2">
                <div class="flex items-center" title="ID">
                  <Hash class="h-4" />
                  <div class="max-md:hidden">ID</div>
                </div>
              </th>
              {#if prefs.debug}
                <th class="border-l p-2">UUID</th>
              {/if}
              <th class="border-l p-2">
                <div class="flex items-center" title="Status">
                  <SquarePen class="h-4" />
                  <div class="max-md:hidden">Status</div>
                </div>
              </th>
              <th class="border-l p-2">Name</th>
              <th class="p-2"></th>
              <th class="border-l p-2">
                <div class="flex items-center" title="Assignee">
                  <UserRoundPlus class="h-4" />
                  <div class="max-md:hidden">Assignee</div>
                </div>
              </th>
              <th class="border-l p-2 max-md:hidden">Reporter</th>
              <th class="border-l max-md:hidden">
                <TaskEstimateHeading />
              </th>
              <th class="relative m-0 w-0 p-0"></th>
            </tr>
          </thead>

          {#each [...planningCtx.nodes].slice(1) as node, index (node.id)}
            <tbody animate:flip={{ duration: 250 }}>
              <!-- eslint-disable-next-line svelte/no-unused-svelte-ignore -->
              <!-- svelte-ignore binding_property_non_reactive -->
              <DagRow bind:this={rows[node.id]} {index} {node} {users} />
            </tbody>
          {/each}
        </table>

        {#if !hideFab}
          <Fab icon={Plus} onclick={insertAction.callback}>
            {insertAction.name}
            {#snippet tooltip()}
              <div class="flex items-center gap-2">
                {insertAction.description}
                {#if insertAction.shortcut}
                  <div class="font-bold">
                    {insertAction.shortcut.toString()}
                  </div>
                {/if}
              </div>
            {/snippet}
          </Fab>
        {/if}
      </div>
    </div>
  {:else}
    <div class="flex items-center justify-center pt-8">
      <div
        class="bg-m3-surface-container flex w-9/12 max-w-[425px] rounded-md border p-4"
      >
        <div class="min-w-16">
          <KosoLogo class="size-16" />
        </div>
        <div class="ml-4">
          <div class="text-md">Welcome to Koso!</div>
          <div class="mt-2 text-sm">
            Koso helps you to organize your work and be productive.
          </div>
          <div class="mt-4">
            <Button variant="filled" icon={ListPlus} onclick={insert}>
              New Task
            </Button>
          </div>
        </div>
      </div>
    </div>
  {/if}
{/await}

<!-- Round the bottom left and right of the table -->
<style>
  :global(.dag-table > tbody:last-child > tr > td:nth-child(2)) {
    border-bottom-left-radius: 0.25rem;
  }

  :global(.dag-table > tbody:last-child > tr > td:nth-last-child(2)) {
    border-bottom-right-radius: 0.25rem;
  }
</style>
