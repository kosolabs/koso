<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/auth.svelte";
  import { Button } from "$lib/components/ui/button";
  import TaskStatus from "$lib/components/ui/task-status/task-status.svelte";
  import { Koso, KosoSocket } from "$lib/dag-table";
  import Navbar from "$lib/navbar.svelte";
  import { fetchProjects, type Project } from "$lib/projects";
  import { cn } from "$lib/utils";
  import type { YTaskProxy } from "$lib/yproxy";
  import { Map } from "immutable";
  import * as Y from "yjs";

  let projects = $state<Project[] | undefined>(undefined);
  fetchProjects().then((resolved) => {
    projects = resolved;
  });

  type KosoPair = {
    project: Project;
    koso: Koso;
    socket: KosoSocket;
  };

  let kosos = $derived(
    Map<string, KosoPair>().withMutations((kosos) => {
      if (projects) {
        for (const project of projects) {
          const koso = new Koso(project.projectId, new Y.Doc());
          const socket = new KosoSocket(koso, project.projectId);
          kosos.set(project.projectId, { project, koso, socket });
        }
      }
    }),
  );

  type ProjectTask = { project: Project; task: YTaskProxy };
  let tasks: ProjectTask[] = $derived.by(() => {
    let tasks = [];
    for (const { project, koso } of kosos.values()) {
      for (const task of koso.tasks) {
        if (task.assignee === auth.user.email && task.status !== "Done") {
          tasks.push({ project, task });
        }
      }
    }
    return tasks;
  });
</script>

<Navbar>
  {#snippet left()}
    <div>
      <h1 class="ml-2 text-lg">Inbox</h1>
    </div>
  {/snippet}
</Navbar>
<div class="p-2">
  <table class="w-full border-separate border-spacing-0 rounded-md border">
    <thead class="text-left text-xs font-bold uppercase">
      <tr>
        <th class="p-2">ID</th>
        <th class="border-l p-2">Status</th>
        <th class="border-l p-2">Project</th>
        <th class="border-l p-2">Name</th>
      </tr>
    </thead>
    <tbody>
      {#each tasks as projectTask}
        <tr
          class={cn(
            "rounded bg-opacity-50 outline outline-2 outline-transparent",
          )}
        >
          <td class={cn("border-t p-2")}>
            {projectTask.task.num}
          </td>
          <td class={cn("border-l border-t px-2")}>
            <TaskStatus task={projectTask.task} />
          </td>
          <td class={cn("border-l border-t px-2")}>
            <Button
              class={cn("p-0")}
              variant="link"
              onclick={() => goto(`/projects/${projectTask.project.projectId}`)}
            >
              {projectTask.project.name}
            </Button>
          </td>
          <td class={cn("border-l border-t p-2")}>
            {projectTask.task.name}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
