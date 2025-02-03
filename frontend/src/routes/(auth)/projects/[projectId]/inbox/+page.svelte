<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { auth } from "$lib/auth.svelte";
  import { Alert } from "$lib/components/ui/alert";
  import { Button } from "$lib/components/ui/button";
  import { Koso, KosoSocket, Node } from "$lib/dag-table";
  import { cn } from "$lib/kosui/utils";
  import { nav } from "$lib/nav.svelte";
  import Navbar from "$lib/navbar.svelte";
  import { fetchProject, type Project } from "$lib/projects";
  import type { YTaskProxy } from "$lib/yproxy";
  import * as Y from "yjs";
  import UnauthorizedModal from "../unauthorized-modal.svelte";
  import TaskAction from "$lib/components/ui/task-action/task-action.svelte";
  import { confetti } from "$lib/components/ui/confetti";
  import { flip } from "svelte/animate";

  const projectId = page.params.projectId;
  nav.lastVisitedProjectId = projectId;

  const koso = new Koso(projectId, new Y.Doc(), isVisible);
  const kosoSocket = new KosoSocket(koso, projectId);
  window.koso = koso;
  window.Y = Y;

  let statusElement: HTMLTableCellElement | undefined = $state();
  let project: Promise<Project> = fetchProject(projectId);

  function getStatusPosition(): DOMRect {
    if (!statusElement) throw new Error("Status element is undefined");
    return statusElement.getBoundingClientRect();
  }

  function isVisible(node: Node): boolean {
    const task = koso.getTask(node.name);
    return task.assignee === auth.user.email && task.status !== "Done";
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
      {#each [...koso.nodes].slice(1) as node, index (node.id)}
        {@const task = koso.getTask(node.name)}
        <tr
          class={cn("bg-opacity-50 rounded outline-2 outline-transparent")}
          animate:flip={{ duration: 250 }}
        >
          <td class={cn("border-t p-2")}>
            {task.num}
          </td>
          <td class={cn("border-t border-l px-2")} bind:this={statusElement}>
            <TaskAction
              {task}
              {koso}
              onOpenChange={() => {
                // TODO: Select row
              }}
              onSelectStatus={(status) => {
                if (status === "Done") confetti.add(getStatusPosition());
                koso.setTaskStatus(node, status, auth.user);
              }}
              onSelectKind={(value) => {
                console.log(`Selected kind ${value}`);
              }}
            />
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
