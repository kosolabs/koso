<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { auth, showUnauthorizedDialog, type User } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { DagTable, Koso, KosoSocket, Node } from "$lib/dag-table";
  import { Alert } from "$lib/kosui/alert";
  import { Button } from "$lib/kosui/button";
  import { nav } from "$lib/nav.svelte";
  import { fetchProject, fetchProjectUsers, type Project } from "$lib/projects";
  import type { YTaskProxy } from "$lib/yproxy";
  import { List } from "immutable";
  import { Notebook } from "lucide-svelte";
  import * as Y from "yjs";

  const projectId = page.params.projectId;
  nav.lastVisitedProjectId = projectId;

  const koso = new Koso(projectId, new Y.Doc(), isVisible, flatten);
  const kosoSocket = new KosoSocket(koso, projectId);
  window.koso = koso;
  window.Y = Y;

  let deflicker: Promise<void> = new Promise((r) => window.setTimeout(r, 50));
  let project: Promise<Project> = fetchProject(projectId);
  let projectUsersPromise: Promise<User[]> = loadProjectUsers();
  let projectUsers: User[] = $state([]);

  async function loadProjectUsers() {
    const users = await fetchProjectUsers(projectId);

    projectUsers = users;
    return projectUsers;
  }

  function isVisible(node: Node): boolean {
    return isTaskVisible(koso.getTask(node.name));
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
    nodes = nodes.push(new Node());

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
    if (kosoSocket.unauthorized) {
      showUnauthorizedDialog();
    }
  });
</script>

<Navbar>
  {#snippet left()}
    <div>
      <h1 class="ml-2 text-lg">
        {#await project}
          Inbox
        {:then project}
          Inbox - {project.name}
        {/await}
      </h1>
    </div>
  {/snippet}
  {#snippet right()}
    <Button
      variant="plain"
      shape="circle"
      tooltip="Project planning view"
      aria-label="Project planning view"
      onclick={() => goto(`/projects/${projectId}`)}
      class="p-2"
    >
      <Notebook size={20} />
    </Button>
  {/snippet}
</Navbar>

{#if kosoSocket.offline}
  <div class="m-2">
    <Alert variant="outlined" color="secondary">
      Connection to server lost. Working offline.
    </Alert>
  </div>
{/if}

{#await projectUsersPromise}
  {#await deflicker}
    <!-- Deflicker load. -->
  {:then}
    <!-- TODO: Make this a Skeleton -->
    <div class="flex flex-col items-center justify-center rounded border p-4">
      <div class="text-l">Loading...</div>
    </div>
  {/await}
{:then}
  <DagTable {koso} users={projectUsers} inboxView={true} />
{/await}
