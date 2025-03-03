<script lang="ts">
  import { page } from "$app/state";
  import { auth, showUnauthorizedDialog, type User } from "$lib/auth.svelte";
  import { DagTable, Koso, KosoSocket, Node } from "$lib/dag-table";
  import { Alert } from "$lib/kosui/alert";
  import { nav } from "$lib/nav.svelte";
  import Navbar from "$lib/navbar.svelte";
  import { fetchProject, fetchProjectUsers, type Project } from "$lib/projects";
  import type { YTaskProxy } from "$lib/yproxy";
  import { List } from "immutable";
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
    // Only show incomplete tasks assigned to this user.
    if (task.assignee !== auth.user.email) {
      return false;
    }
    const progress = koso.getProgress(task.id);
    if (progress.isComplete()) {
      return false;
    }
    // Only show actionable juggled tasks.
    if (task.kind === "Juggled") {
      if (task.status === "Done") {
        return false;
      }
      const blocked = progress.done !== progress.total - 1;
      return !blocked;
    }

    return true;
  }

  function flatten(): List<Node> {
    const parents = koso.parents;
    let nodes: List<Node> = List();
    nodes = nodes.push(new Node());

    for (const task of koso.tasks) {
      if (isTaskVisible(task)) {
        // Walk up the tree to craft the full path.
        let id = "";
        let parent = parents.get(task.id);
        while (parent) {
          let parentId = parent[0];
          id += `${parentId}/`;
          parent = parents.get(parentId);
        }
        id += task.id;
        nodes = nodes.push(Node.parse(id));
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
  <DagTable {koso} users={projectUsers} extraActions={[]} inboxView={true} />
{/await}
