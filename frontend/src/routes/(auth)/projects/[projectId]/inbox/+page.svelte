<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth, showUnauthorizedDialog } from "$lib/auth.svelte";
  import { command, type ActionID } from "$lib/components/ui/command-palette";
  import { Navbar } from "$lib/components/ui/navbar";
  import { DagTable, Node } from "$lib/dag-table";
  import OfflineAlert from "$lib/dag-table/offline-alert.svelte";
  import { newPlanningContext } from "$lib/dag-table/planning-context.svelte";
  import { getProjectContext } from "$lib/dag-table/project-context.svelte";
  import { Button } from "$lib/kosui/button";
  import { Action } from "$lib/kosui/command";
  import type { YTaskProxy } from "$lib/yproxy";
  import { List } from "immutable";
  import {
    Notebook,
    PanelTopClose,
    PanelTopOpen,
    SquarePen,
  } from "lucide-svelte";
  import { onMount } from "svelte";

  const project = getProjectContext();
  const inbox = newPlanningContext(project.koso, isVisible, flatten);
  const { koso } = inbox;

  function isVisible(taskId: string): boolean {
    return isTaskVisible(koso.getTask(taskId));
  }

  function isTaskVisible(task: YTaskProxy): boolean {
    // Don't show tasks not assigned to the user
    if (task.assignee !== null && task.assignee !== auth.user.email) {
      return false;
    }
    // Don't show rollup tasks where every child is assigned.
    if (
      task.yKind === null &&
      task.children.length > 0 &&
      Array.from(task.children.slice())
        .map((childId) => koso.getTask(childId))
        .every(
          (child) =>
            child.assignee !== null || koso.getProgress(child.id).isComplete(),
        )
    ) {
      return false;
    }

    // Don't show unassigned task where none of the parents are assigned to the user
    if (
      task.assignee === null &&
      koso
        .getParents(task.id)
        .filter((parent) => parent.yKind === null)
        .every((parent) => parent.assignee !== auth.user.email)
    ) {
      return false;
    }
    const progress = koso.getProgress(task.id);
    return !progress.isComplete() && !progress.isBlocked();
  }

  function flatten(): List<Node> {
    const parents = koso.parents;
    let nodes: List<Node> = List();
    nodes = nodes.push(inbox.root);

    for (const task of koso.tasks) {
      if (task.id !== "root" && isTaskVisible(task)) {
        // Walk up the tree to craft the full path.
        let parent = parents.get(task.id);
        const path = [task.id];
        while (parent) {
          let parentId = parent[0];
          path.unshift(parentId);
          parent = parents.get(parentId);
        }
        // We omit the leading 'root' id from the node path.
        if (path[0] === "root") {
          path.shift();
        }
        nodes = nodes.push(new Node({ path: List.of(...path) }));
      }
    }

    return nodes;
  }

  $effect(() => {
    if (project.socket.unauthorized) {
      showUnauthorizedDialog();
    }
  });

  onMount(() => {
    const actions: Action<ActionID>[] = [
      new Action({
        id: "DetailPanelClose",
        callback: () => (inbox.detailPanel = "none"),
        title: "Close task description",
        description: "Close / hide the task description markdown panel",
        icon: PanelTopClose,
      }),
      new Action({
        id: "DetailPanelViewer",
        callback: () => (inbox.detailPanel = "view"),
        title: "View task description",
        description: "Open / show the task description markdown viewer",
        icon: PanelTopOpen,
        enabled: () => !!inbox.selected,
      }),
      new Action({
        id: "DetailPanelEditor",
        callback: () => (inbox.detailPanel = "edit"),
        title: "Edit task description",
        description: "Open / show the task description markdown editor",
        icon: SquarePen,
        enabled: () =>
          !!inbox.selected && inbox.koso.isEditable(inbox.selected.name),
      }),
    ];

    return command.register(...actions);
  });
</script>

<Navbar>
  {#snippet left()}
    <div>
      <h1 class="ml-2 text-lg">
        Inbox - {project.name}
      </h1>
    </div>
  {/snippet}
  {#snippet right()}
    <Button
      variant="plain"
      shape="circle"
      tooltip="Project planning view"
      aria-label="Project planning view"
      onclick={() => goto(`/projects/${project.id}`)}
      class="p-2"
    >
      <Notebook size={20} />
    </Button>
  {/snippet}
</Navbar>

<OfflineAlert offline={project.socket.offline} />

<DagTable users={project.users} inboxView={true} />
