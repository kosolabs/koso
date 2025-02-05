<script lang="ts">
  import { page } from "$app/state";
  import { auth, type User } from "$lib/auth.svelte";
  import { Alert } from "$lib/components/ui/alert";
  import { Button } from "$lib/components/ui/button";
  import { DagTable, Koso, KosoSocket, Node } from "$lib/dag-table";
  import { cn } from "$lib/kosui/utils";
  import { nav } from "$lib/nav.svelte";
  import Navbar from "$lib/navbar.svelte";
  import { fetchProject, fetchProjectUsers, type Project } from "$lib/projects";
  import type { YTaskProxy } from "$lib/yproxy";
  import * as Y from "yjs";
  import UnauthorizedModal from "../unauthorized-modal.svelte";
  import { List } from "immutable";

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
    return task.assignee === auth.user.email && task.status !== "Done";
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
  <div class="m-4">
    <Alert>Connection to server lost. Working offline.</Alert>
  </div>
{/if}

<UnauthorizedModal open={kosoSocket.unauthorized} />

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
  <div class="p-2">
    <DagTable {koso} users={projectUsers} extraActions={[]} inboxView={true} />
  </div>
{/await}
