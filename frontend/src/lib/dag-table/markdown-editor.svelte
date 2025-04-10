<script lang="ts">
  import { CodeMirror } from "$lib/components/ui/code-mirror";
  import { Button } from "$lib/kosui/button";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { Pencil, Trash, X } from "lucide-svelte";
  import { toast } from "svelte-sonner";
  import { slide } from "svelte/transition";
  import * as Y from "yjs";
  import MarkdownViewer from "./markdown-viewer.svelte";
  import { getProjectContext } from "./project-context.svelte";

  type Props = {
    taskId: string | undefined;
  };
  let { taskId }: Props = $props();

  const project = getProjectContext();

  let task = $derived(taskId ? project.koso.getTask(taskId) : undefined);
  let yDesc: Y.Text | undefined = $state.raw();
  let desc: string | undefined = $state.raw();

  function hideDetails() {
    project.koso.detailPanel = "none";
  }

  function viewDetails() {
    project.koso.detailPanel = "view";
  }

  function editDetails() {
    if (taskId && !project.koso.isEditable(taskId)) {
      toast.warning(
        "This is a managed task and cannot be edited in Koso directly.",
      );
      return;
    }
    yDesc = task?.getOrNewDesc();
    project.koso.detailPanel = "edit";
  }

  function deleteDetails() {
    yDesc = undefined;
    task?.delDesc();
  }

  function handleKeyDownEditing(event: KeyboardEvent) {
    if (Shortcut.ESCAPE.matches(event)) {
      viewDetails();
    }
    event.stopImmediatePropagation();
  }

  $effect(() => {
    yDesc = task?.desc || undefined;
    desc = yDesc?.toString();
    return task?.observeDeep(() => {
      yDesc = task.desc || undefined;
      desc = yDesc?.toString();
    });
  });

  $effect(() => {
    if (
      taskId &&
      !project.koso.isEditable(taskId) &&
      project.koso.detailPanel === "edit"
    ) {
      viewDetails();
    }
  });
</script>

{#if project.koso.detailPanel !== "none"}
  <div class="relative mb-2 rounded-md border" transition:slide>
    <div
      class="flex items-center p-2 text-lg font-extralight"
      role="heading"
      aria-level="1"
      ondblclick={editDetails}
    >
      {task?.name || "No task selected"}
      <div class="top-2 right-2 ml-auto flex gap-1">
        {#if task && project.koso.isEditable(task.id)}
          <Button icon={Pencil} variant="plain" onclick={editDetails} />
        {/if}
        {#if yDesc !== undefined}
          <Button icon={Trash} variant="plain" onclick={deleteDetails} />
        {/if}
        <Button icon={X} variant="plain" onclick={hideDetails} />
      </div>
    </div>
    <hr />
    <div
      class="h-96 max-h-96 overflow-scroll"
      role="document"
      ondblclick={editDetails}
    >
      {#if yDesc && task && project.koso.isEditable(task.id) && project.koso.detailPanel === "edit"}
        <CodeMirror yText={yDesc} onkeydown={handleKeyDownEditing} />
      {:else if desc}
        <MarkdownViewer class="p-2" value={desc} />
      {/if}
    </div>
  </div>
{/if}
