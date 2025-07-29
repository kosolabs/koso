<script lang="ts">
  import { page } from "$app/state";
  import { AnthropicStream } from "$lib/anthropic.svelte";
  import { headers } from "$lib/api";
  import { getAuthContext } from "$lib/auth.svelte";
  import { MarkdownViewer } from "$lib/components/ui/markdown-viewer";
  import { Navbar } from "$lib/components/ui/navbar";
  import { DagTable, newPlanningContext } from "$lib/dag-table";
  import OfflineAlert from "$lib/dag-table/offline-alert.svelte";
  import type { Node } from "$lib/dag-table/planning-context.svelte";
  import { getProjectContext } from "$lib/dag-table/project-context.svelte";
  import {
    Markdown,
    MarkdownBlockquote,
    MarkdownCode,
    MarkdownHeading,
    MarkdownLink,
    MarkdownList,
    MarkdownTable,
    MarkdownTableCell,
  } from "$lib/kosui/markdown";
  import { CircularProgress } from "$lib/kosui/progress";
  import { Tooltip } from "$lib/kosui/tooltip";
  import { List } from "immutable";
  import { twMerge } from "tailwind-merge";

  const projectId = page.params.projectId;
  const taskId = page.params.taskId;
  if (!projectId) throw new Error("Missing projectId slug");
  if (!taskId) throw new Error("Missing taskId slug");
  const simulate = page.url.searchParams.get("simulate") === "true";

  const { koso, socket, name, users } = getProjectContext();
  const auth = getAuthContext();
  const planningCtx = newPlanningContext(koso, taskId);

  let offline: boolean = $derived(socket.offline);

  let summary = new AnthropicStream();
  summary.fetch(
    `/api/anthropic/summarize?projectId=${projectId}&taskId=${taskId}&simulate=${simulate}&model=claude-sonnet-4-20250514`,
    {
      method: "GET",
      headers: headers(auth),
    },
  );

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

  let task = $derived(koso.getTask(taskId));
  let progress = $derived.by(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    koso.events;
    return koso.getProgress(taskId);
  });
  let paths = $derived(getLongestPaths());
</script>

<Navbar>
  {#snippet left()}
    <h1 class="ml-2 text-lg">
      {name} Dashboard
    </h1>
  {/snippet}
</Navbar>

<OfflineAlert {offline} />

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
        <div class="text-xl font-extralight">
          Due Date: {formatDate(task.deadline)} ({Math.floor(
            (task.deadline - Date.now()) / 86400000,
          )} days)
        </div>
        <h2 class="text-xl font-extralight">
          Remaining Estimate: {progress.remainingEstimate} points
        </h2>
      </div>
    </div>
  {/if}
  <h2 class="flex items-center gap-2 text-2xl font-extralight">
    {#if summary.running}
      <CircularProgress class="text-m3-primary" />
      Koso Agent is summarizing the iteration...
    {:else}
      Koso Agent Summary
    {/if}
  </h2>
  <hr />
  <div class="flex flex-col gap-2 rounded-md border p-2">
    <Markdown
      value={summary.stream.join("")}
      options={{ breaks: true, gfm: true }}
    >
      {#snippet blockquote(props)}
        <MarkdownBlockquote class="border border-l-4 p-2" {...props} />
      {/snippet}
      {#snippet code(props)}
        <MarkdownCode class="rounded border p-2 text-sm" {...props} />
      {/snippet}
      {#snippet heading({ token, children })}
        <MarkdownHeading
          class={twMerge(
            token.depth === 1 && "text-2xl font-extralight",
            token.depth === 2 && "text-xl font-extralight",
          )}
          {token}
          {children}
        />
      {/snippet}
      {#snippet list({ token, children })}
        <MarkdownList
          class={twMerge("ml-4", token.ordered ? "list-decimal" : "list-disc")}
          {token}
          {children}
        />
      {/snippet}
      {#snippet table(props)}
        <MarkdownTable class="w-min" {...props} />
      {/snippet}
      {#snippet tableCell(props)}
        <MarkdownTableCell class="border p-1 whitespace-nowrap" {...props} />
      {/snippet}
      {#snippet link(props)}
        <MarkdownLink
          class="text-m3-primary underline hover:opacity-80"
          {...props}
        />
      {/snippet}
    </Markdown>
  </div>

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
