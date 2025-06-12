<script module lang="ts">
  type Status = "On Track" | "At Risk";
</script>

<script lang="ts">
  import { page } from "$app/state";
  import { MarkdownViewer } from "$lib/components/ui/markdown-viewer";
  import { Navbar } from "$lib/components/ui/navbar";
  import { getPrefsContext } from "$lib/components/ui/prefs";
  import { DagTable, newPlanningContext } from "$lib/dag-table";
  import type { Progress } from "$lib/dag-table/koso.svelte";
  import OfflineAlert from "$lib/dag-table/offline-alert.svelte";
  import type { Node } from "$lib/dag-table/planning-context.svelte";
  import { getProjectContext } from "$lib/dag-table/project-context.svelte";
  import { Alert } from "$lib/kosui/alert";
  import type { colors } from "$lib/kosui/base";
  import CircularProgress from "$lib/kosui/progress/circular-progress.svelte";
  import { Tooltip } from "$lib/kosui/tooltip";
  import { List } from "immutable";

  const taskId = page.params.taskId;

  const prefs = getPrefsContext();
  const { koso, socket, name, users } = getProjectContext();
  const planningCtx = newPlanningContext(koso, prefs, taskId);

  function getStatusColor(status: Status): (typeof colors)[number] {
    if (status === "On Track") {
      return "primary";
    } else if (status === "At Risk") {
      return "error";
    } else {
      return "tertiary";
    }
  }

  function getLongestPaths() {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    koso.events;
    const allPaths = getAllLeaves(planningCtx.root)
      .sortBy((node) => node.length)
      .reverse();
    const longestPath = allPaths.get(0);
    if (!longestPath) {
      return allPaths;
    }
    return allPaths.filter((node) => node.length === longestPath.length);
  }

  function getAllLeaves(node: Node, paths: List<Node> = List()): List<Node> {
    const task = koso.getTask(node.name);
    if (task.yStatus === "Done") {
      return paths;
    }
    if (task.children.length === 0) {
      return paths.push(node);
    }
    task.children.forEach((name) => {
      paths = getAllLeaves(node.child(name), paths);
    });
    return paths;
  }

  function formatDate(ts: number) {
    const date = new Date(ts);
    return date.toLocaleDateString();
  }

  function getStatus(deadline: number, progress: Progress): Status {
    const remainingDays = Math.floor((deadline - Date.now()) / 86400000);
    return progress.remainingEstimate! / 2 < remainingDays
      ? "On Track"
      : "At Risk";
  }

  let task = $derived(koso.getTask(taskId));
  let progress = $derived.by(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    koso.events;
    return koso.getProgress(taskId);
  });
  let status = $derived(getStatus($task.deadline!, progress));
  let paths = $derived(getLongestPaths());
</script>

<Navbar>
  {#snippet left()}
    <h1 class="ml-2 text-lg">
      {name} Dashboard
    </h1>
  {/snippet}
</Navbar>

<OfflineAlert offline={socket.offline} />

<div class="flex flex-col gap-2 p-2">
  <h1 class="text-3xl font-extralight">{task.name}</h1>
  <hr />

  {#if task.deadline}
    <h2 class="gap-2 text-2xl font-extralight">Status</h2>
    <hr />
    <div class="flex flex-wrap items-center gap-2 rounded-md border p-2">
      <Tooltip arrow rich>
        {#snippet trigger({ ref, ...props })}
          <button use:ref {...props}>
            <CircularProgress
              class="text-m3-primary"
              size="100"
              progress={progress.done / progress.total}
            >
              {Math.round((progress.done * 100) / progress.total)}%
            </CircularProgress>
          </button>
        {/snippet}
        <h2 class="text-xl font-extralight">
          Progress: {progress.done} / {progress.total}
        </h2>
      </Tooltip>
      <div class="flex flex-col items-start">
        <Alert
          class="py-1 text-xl font-extralight"
          variant="filled"
          color={getStatusColor(status)}
        >
          {status}
        </Alert>

        <div class="text-xl font-extralight">
          Due Date: {formatDate(task.deadline)} ({Math.floor(
            (task.deadline - Date.now()) / 86400000,
          )} days)
        </div>
        <h2 class="text-xl font-extralight">
          Remaining Estimate: {progress.remainingEstimate} points ({(progress.remainingEstimate ??
            0) / 2} days)
        </h2>
      </div>
    </div>
  {/if}

  <h2 class="text-2xl font-extralight">Critical Paths</h2>
  <hr />
  <div class="rounded-md border p-2">
    {#each paths as node, index (node)}
      <h3 class="font-extralight">Path {index + 1}:</h3>
      {#each node.path as taskId, index (taskId)}
        {@const task = koso.getTask(taskId)}
        <div style:margin-left={`${20 * index}px`}>➜ {task.name}</div>
      {/each}
    {/each}
  </div>

  {#if task.desc}
    <h2 class="text-2xl font-extralight">Description</h2>
    <hr />
    <div class="rounded-md border p-2">
      <MarkdownViewer value={task.desc.toString()} />
    </div>
  {/if}

  <h2 class="text-2xl font-extralight">Tasks</h2>
  <hr />
  <DagTable {users} hideFab />
</div>
