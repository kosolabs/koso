<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { auth } from "$lib/auth.svelte";
  import { Alert } from "$lib/components/ui/alert";
  import { Button } from "$lib/components/ui/button";
  import TaskStatus from "$lib/components/ui/task-status/task-status.svelte";
  import { Koso, KosoSocket } from "$lib/dag-table";
  import { cn } from "$lib/kosui/utils";
  import { nav } from "$lib/nav.svelte";
  import Navbar from "$lib/navbar.svelte";
  import { fetchProject, type Project } from "$lib/projects";
  import type { YTaskProxy } from "$lib/yproxy";
  import * as Y from "yjs";
  import UnauthorizedModal from "../unauthorized-modal.svelte";

  const projectId = page.params.projectId;
  nav.lastVisitedProjectId = projectId;

  const koso = new Koso(projectId, new Y.Doc());
  const kosoSocket = new KosoSocket(koso, projectId);
  window.koso = koso;
  window.Y = Y;

  let project: Promise<Project> = fetchProject(projectId);

  let tasks: YTaskProxy[] = $derived.by(() => {
    let tasks = [];
    for (const task of koso.tasks) {
      if (task.assignee === auth.user.email && task.status !== "Done") {
        tasks.push(task);
      }
    }
    return tasks;
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
  <div class="m-4">
    <Alert>Connection to server lost. Working offline.</Alert>
  </div>
{/if}

<UnauthorizedModal open={kosoSocket.unauthorized} />

<div class="p-2">
  <table class="w-full border-separate border-spacing-0 rounded-md border">
    <thead class="text-left text-xs font-bold uppercase">
      <tr>
        <th class="p-2">ID</th>
        <th class="border-l p-2">Status</th>
        <th class="border-l p-2">Name</th>
      </tr>
    </thead>
    <tbody>
      {#each tasks as task}
        <tr class={cn("bg-opacity-50 rounded outline-2 outline-transparent")}>
          <td class={cn("border-t p-2")}>
            {task.num}
          </td>
          <td class={cn("border-t border-l px-2")}>
            <TaskStatus {task} />
          </td>
          <td class={cn("border-t border-l px-2")}>
            <Button
              class={cn("p-0")}
              variant="link"
              onclick={() => {
                sessionStorage.setItem("taskId", task.id);
                goto(`/projects/${projectId}`);
              }}
            >
              {task.name}
            </Button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
