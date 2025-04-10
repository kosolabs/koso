<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth, showUnauthorizedDialog } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { DagTable, Node } from "$lib/dag-table";
  import OfflineAlert from "$lib/dag-table/offline-alert.svelte";
  import { newPlanningContext } from "$lib/dag-table/planning-context.svelte";
  import { getProjectContext } from "$lib/dag-table/project-context.svelte";
  import { Button } from "$lib/kosui/button";
  import type { YTaskProxy } from "$lib/yproxy";
  import { List } from "immutable";
  import { Notebook } from "lucide-svelte";

  const project = getProjectContext();
  const inbox = newPlanningContext(project, isVisible, flatten);

  function isVisible(taskId: string): boolean {
    return isTaskVisible(inbox.koso.getTask(taskId));
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
        .map((childId) => inbox.koso.getTask(childId))
        .every(
          (child) =>
            child.assignee !== null ||
            inbox.koso.getProgress(child.id).isComplete(),
        )
    ) {
      return false;
    }

    // Don't show unassigned task where none of the parents are assigned to the user
    if (
      task.assignee === null &&
      inbox.koso
        .getParents(task.id)
        .filter((parent) => parent.yKind === null)
        .every((parent) => parent.assignee !== auth.user.email)
    ) {
      return false;
    }
    const progress = inbox.koso.getProgress(task.id);
    return !progress.isComplete() && !progress.isBlocked();
  }

  function flatten(): List<Node> {
    const parents = inbox.koso.parents;
    let nodes: List<Node> = List();
    nodes = nodes.push(inbox.koso.root);

    for (const task of inbox.koso.tasks) {
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
    if (inbox.projectCtx.socket.unauthorized) {
      showUnauthorizedDialog();
    }
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

<OfflineAlert offline={inbox.projectCtx.socket.offline} />

<DagTable planningCtx={inbox} users={project.users} inboxView={true} />
