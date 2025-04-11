<script lang="ts">
  import { CodeMirror } from "$lib/components/ui/code-mirror";
  import { Button } from "$lib/kosui/button";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { Pencil, Trash, X } from "lucide-svelte";
  import { tick } from "svelte";
  import { toast } from "svelte-sonner";
  import { slide } from "svelte/transition";
  import * as Y from "yjs";
  import type { DetailPanelStates } from "./koso.svelte";
  import MarkdownViewer from "./markdown-viewer.svelte";
  import { getProjectContext } from "./project-context.svelte";

  interface DetailPanelRenderer {
    detailPanel: DetailPanelStates;
  }

  type Props = {
    taskId: string | undefined;
    detailPanelRenderer: DetailPanelRenderer;
  };
  let { taskId, detailPanelRenderer }: Props = $props();
  let editor: CodeMirror | undefined = $state();

  const { koso } = getProjectContext();

  let task = $derived(taskId ? koso.getTask(taskId) : undefined);
  let yDesc: Y.Text | undefined = $state.raw();
  let desc: string | undefined = $state.raw();

  function hideDetails() {
    detailPanelRenderer.detailPanel = "none";
  }

  function viewDetails() {
    detailPanelRenderer.detailPanel = "view";
  }

  function editDetails() {
    if (taskId && !koso.isEditable(taskId)) {
      toast.warning(
        "This is a managed task and cannot be edited in Koso directly.",
      );
      return;
    }
    yDesc = task?.getOrNewDesc();
    detailPanelRenderer.detailPanel = "edit";
    // Focus the editor after it gets rendered (after a tick)
    tick().then(() => editor?.focus());
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
      !koso.isEditable(taskId) &&
      detailPanelRenderer.detailPanel === "edit"
    ) {
      viewDetails();
    }
  });
</script>

{#if detailPanelRenderer.detailPanel !== "none"}
  <div class="relative mb-2 rounded-md border" transition:slide>
    <div
      class="flex items-center p-2 text-lg font-extralight"
      role="heading"
      aria-level="1"
      ondblclick={editDetails}
    >
      {$task?.name || "No task selected"}
      <div class="top-2 right-2 ml-auto flex gap-1">
        {#if taskId && koso.isEditable(taskId)}
          <Button
            aria-label="Edit task description"
            icon={Pencil}
            variant="plain"
            onclick={editDetails}
          />
        {/if}
        {#if yDesc !== undefined}
          <Button
            aria-label="Delete task description"
            icon={Trash}
            variant="plain"
            onclick={deleteDetails}
          />
        {/if}
        <Button
          aria-label="Hide task description panel"
          icon={X}
          variant="plain"
          onclick={hideDetails}
        />
      </div>
    </div>
    <hr />
    <div
      class="h-96 max-h-96 overflow-scroll"
      role="document"
      ondblclick={editDetails}
    >
      {#if yDesc && taskId && koso.isEditable(taskId) && detailPanelRenderer.detailPanel === "edit"}
        <CodeMirror
          bind:this={editor}
          yText={yDesc}
          onkeydown={handleKeyDownEditing}
        />
      {:else if desc}
        <MarkdownViewer class="p-2" value={desc} />
      {/if}
    </div>
  </div>
{/if}
